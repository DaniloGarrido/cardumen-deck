use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Mutex;

use once_cell::sync::Lazy;
use serde::Serialize;
use thiserror::Error;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("La carpeta no pertenece a un repositorio Git: {0}")]
    NotRepository(String),
    #[error("Hay cambios locales pendientes: {0}")]
    DirtyWorkTree(String),
    #[error("Nombre de rama inválido: {0}")]
    InvalidBranch(String),
    #[error("La rama no existe: {0}")]
    BranchNotFound(String),
    #[error("La rama actual no tiene upstream configurado")]
    NoUpstream,
    #[error("El repositorio no tiene remotos configurados")]
    NoRemote,
    #[error("Hay una operación Git en curso: {0}")]
    OperationInProgress(String),
    #[error("No se puede operar sobre la rama actual: {0}")]
    CurrentBranch(String),
    #[error("git: {0}")]
    Command(String),
    #[error("No se pudo ejecutar git: {0}")]
    Spawn(String),
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GitBranchInfo {
    pub name: String,
    pub kind: String,
    pub current: bool,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GitRepositorySummary {
    pub path: String,
    pub name: String,
    pub branch: Option<String>,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub dirty: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GitState {
    pub path: String,
    pub root: String,
    pub current_branch: Option<String>,
    pub detached: bool,
    pub upstream: Option<String>,
    pub ahead: usize,
    pub behind: usize,
    pub dirty: bool,
    pub changed_files: Vec<String>,
    pub untracked_files: Vec<String>,
    pub conflict_files: Vec<String>,
    pub branches: Vec<GitBranchInfo>,
    pub remotes: Vec<String>,
    pub operation: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitOperationResult {
    pub success: bool,
    pub action: String,
    pub message: String,
    pub stdout: String,
    pub stderr: String,
    pub state: GitState,
}

#[derive(Debug)]
struct GitOutput {
    success: bool,
    stdout: String,
    stderr: String,
}

static GIT_WRITE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn git_command(path: &str, args: &[&str]) -> Command {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(path)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "never")
        .env("GIT_MERGE_AUTOEDIT", "no")
        .env("GIT_EDITOR", "true")
        .env("GIT_SEQUENCE_EDITOR", "true")
        .env("GIT_PAGER", "cat")
        .env("PAGER", "cat")
        .env("GIT_HTTP_LOW_SPEED_LIMIT", "1")
        .env("GIT_HTTP_LOW_SPEED_TIME", "60")
        .env("LANG", "C")
        .env("LC_ALL", "C");

    #[cfg(target_os = "windows")]
    command.creation_flags(0x08000000);

    command
}

fn run_git(path: &str, args: &[&str]) -> Result<GitOutput, GitError> {
    let output = git_command(path, args)
        .output()
        .map_err(|error| GitError::Spawn(error.to_string()))?;

    Ok(GitOutput {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

fn run_git_checked(path: &str, args: &[&str]) -> Result<GitOutput, GitError> {
    let output = run_git(path, args)?;
    if output.success {
        Ok(output)
    } else {
        Err(GitError::Command(command_error(&output)))
    }
}

fn command_error(output: &GitOutput) -> String {
    let stderr = output.stderr.trim();
    if !stderr.is_empty() {
        return stderr.to_string();
    }
    let stdout = output.stdout.trim();
    if !stdout.is_empty() {
        return stdout.to_string();
    }
    "Git devolvió un error sin detalle".to_string()
}

fn output_message(output: &GitOutput) -> String {
    if output.success {
        let stdout = output.stdout.trim();
        if stdout.is_empty() {
            "Operación completada".to_string()
        } else {
            stdout.lines().last().unwrap_or(stdout).to_string()
        }
    } else {
        command_error(output)
    }
}

fn truncate_output(value: &str) -> String {
    const LIMIT: usize = 12_000;
    if value.chars().count() <= LIMIT {
        return value.trim().to_string();
    }
    let mut truncated: String = value.chars().take(LIMIT).collect();
    truncated.push_str("\n… salida truncada");
    truncated.trim().to_string()
}

fn repo_root(path: &str) -> Result<String, GitError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(GitError::NotRepository(path.to_string()));
    }

    let output = run_git(trimmed, &["rev-parse", "--show-toplevel"])?;
    if !output.success {
        return Err(GitError::NotRepository(trimmed.to_string()));
    }

    let root = output.stdout.trim();
    if root.is_empty() {
        return Err(GitError::NotRepository(trimmed.to_string()));
    }

    Ok(root.to_string())
}

fn parse_track(track: &str) -> (usize, usize) {
    let mut ahead = 0;
    let mut behind = 0;
    let inner = track.trim().trim_start_matches('[').trim_end_matches(']');

    for part in inner.split(',') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("ahead ") {
            ahead = value.parse().unwrap_or(0);
        } else if let Some(value) = part.strip_prefix("behind ") {
            behind = value.parse().unwrap_or(0);
        }
    }

    (ahead, behind)
}

fn branches_for_root(root: &str) -> Result<Vec<GitBranchInfo>, GitError> {
    let output = run_git_checked(
        root,
        &[
            "for-each-ref",
            "--format=%(refname:short)%09%(refname)%09%(upstream:short)%09%(upstream:track)%09%(HEAD)",
            "refs/heads",
            "refs/remotes",
        ],
    )?;

    let mut branches = Vec::new();
    for line in output.stdout.lines() {
        let mut fields = line.split('\t');
        let name = fields.next().unwrap_or_default().trim();
        let refname = fields.next().unwrap_or_default().trim();
        let upstream = fields.next().unwrap_or_default().trim();
        let track = fields.next().unwrap_or_default().trim();
        let head = fields.next().unwrap_or_default().trim();

        if name.is_empty() || refname.is_empty() || name == "HEAD" || name.ends_with("/HEAD") {
            continue;
        }

        let (ahead, behind) = parse_track(track);
        branches.push(GitBranchInfo {
            name: name.to_string(),
            kind: if refname.starts_with("refs/remotes/") {
                "remote".to_string()
            } else {
                "local".to_string()
            },
            current: head == "*",
            upstream: if upstream.is_empty() {
                None
            } else {
                Some(upstream.to_string())
            },
            ahead,
            behind,
        });
    }

    branches.sort_by(|left, right| {
        right
            .current
            .cmp(&left.current)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.name.cmp(&right.name))
    });

    Ok(branches)
}

fn status_files(root: &str) -> Result<(Vec<String>, Vec<String>), GitError> {
    let output = run_git_checked(
        root,
        &["status", "--porcelain=v1", "--untracked-files=normal"],
    )?;
    let mut changed = Vec::new();
    let mut untracked = Vec::new();

    for line in output.stdout.lines() {
        let path = line.get(3..).unwrap_or(line).trim();
        if path.is_empty() {
            continue;
        }
        if line.starts_with("??") {
            untracked.push(path.to_string());
        } else {
            changed.push(path.to_string());
        }
    }

    Ok((changed, untracked))
}

fn conflict_files(root: &str) -> Result<Vec<String>, GitError> {
    let output = run_git(root, &["diff", "--name-only", "--diff-filter=U"])?;
    if !output.success {
        return Ok(Vec::new());
    }
    Ok(output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect())
}

fn remotes_for_root(root: &str) -> Result<Vec<String>, GitError> {
    let output = run_git_checked(root, &["remote"])?;
    Ok(output
        .stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect())
}

fn marker_exists(root: &str, marker: &str) -> bool {
    let output = match run_git(
        root,
        &["rev-parse", "--path-format=absolute", "--git-path", marker],
    ) {
        Ok(output) => output,
        Err(_) => return false,
    };

    if !output.success {
        return false;
    }

    let path = output.stdout.trim();
    !path.is_empty() && Path::new(path).exists()
}

fn operation_for_root(root: &str) -> Option<String> {
    if marker_exists(root, "MERGE_HEAD") {
        return Some("merge".to_string());
    }
    if marker_exists(root, "rebase-merge") || marker_exists(root, "rebase-apply") {
        return Some("rebase".to_string());
    }
    if marker_exists(root, "CHERRY_PICK_HEAD") {
        return Some("cherry-pick".to_string());
    }
    if marker_exists(root, "REVERT_HEAD") {
        return Some("revert".to_string());
    }
    None
}

fn state_for_root(root: &str) -> Result<GitState, GitError> {
    let branches = branches_for_root(root)?;
    let current = branches.iter().find(|branch| branch.current);
    let (changed_files, untracked_files) = status_files(root)?;
    let conflict_files = conflict_files(root)?;
    let remotes = remotes_for_root(root)?;
    let operation = operation_for_root(root);

    Ok(GitState {
        path: root.to_string(),
        root: root.to_string(),
        current_branch: current.map(|branch| branch.name.clone()),
        detached: current.is_none(),
        upstream: current.and_then(|branch| branch.upstream.clone()),
        ahead: current.map(|branch| branch.ahead).unwrap_or(0),
        behind: current.map(|branch| branch.behind).unwrap_or(0),
        dirty: !changed_files.is_empty(),
        changed_files,
        untracked_files,
        conflict_files,
        branches,
        remotes,
        operation,
    })
}

pub fn state_for(path: &str) -> Result<GitState, GitError> {
    let root = repo_root(path)?;
    state_for_root(&root)
}

fn summary_for_root(root: &str) -> Result<GitRepositorySummary, GitError> {
    let branches = branches_for_root(root)?;
    let current = branches.iter().find(|branch| branch.current);
    let (changed, _) = status_files(root)?;
    let dirty = !changed.is_empty();
    let name = Path::new(root)
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or(root)
        .to_string();

    Ok(GitRepositorySummary {
        path: root.to_string(),
        name,
        branch: current.map(|branch| branch.name.clone()),
        upstream: current.and_then(|branch| branch.upstream.clone()),
        ahead: current.map(|branch| branch.ahead).unwrap_or(0),
        behind: current.map(|branch| branch.behind).unwrap_or(0),
        dirty,
    })
}

pub fn repositories_for(paths: &[String]) -> Vec<GitRepositorySummary> {
    let mut seen = HashSet::new();
    let mut repositories = Vec::new();

    for path in paths {
        let root = match repo_root(path) {
            Ok(root) => root,
            Err(_) => continue,
        };
        let key = root.to_lowercase();
        if !seen.insert(key) {
            continue;
        }
        if let Ok(summary) = summary_for_root(&root) {
            repositories.push(summary);
        }
    }

    repositories.sort_by_key(|repository| repository.name.to_lowercase());
    repositories
}

pub fn branches_for(paths: &[String]) -> HashMap<String, String> {
    let mut branches = HashMap::new();

    for path in paths {
        let path = path.trim();
        if path.is_empty() {
            continue;
        }
        if let Ok(root) = repo_root(path) {
            if let Ok(state) = state_for_root(&root) {
                if let Some(branch) = state.current_branch {
                    branches.insert(path.to_string(), branch);
                }
            }
        }
    }

    branches
}

fn with_write_lock<T>(operation: impl FnOnce() -> Result<T, GitError>) -> Result<T, GitError> {
    let _guard = GIT_WRITE_LOCK
        .lock()
        .map_err(|_| GitError::Command("No se pudo bloquear la operación Git".to_string()))?;
    operation()
}

fn validate_branch(root: &str, name: &str) -> Result<String, GitError> {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.starts_with('-') {
        return Err(GitError::InvalidBranch(name.to_string()));
    }

    let output = run_git(root, &["check-ref-format", "--branch", trimmed])?;
    if !output.success {
        return Err(GitError::InvalidBranch(trimmed.to_string()));
    }

    Ok(trimmed.to_string())
}

fn find_branch<'a>(state: &'a GitState, name: &str) -> Option<&'a GitBranchInfo> {
    state.branches.iter().find(|branch| branch.name == name)
}

fn ensure_clean(state: &GitState) -> Result<(), GitError> {
    if !state.dirty {
        return Ok(());
    }

    let files = if state.changed_files.is_empty() {
        "archivos modificados".to_string()
    } else {
        state
            .changed_files
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    };
    Err(GitError::DirtyWorkTree(files))
}

fn ensure_no_operation(state: &GitState) -> Result<(), GitError> {
    if let Some(operation) = state.operation.as_deref() {
        return Err(GitError::OperationInProgress(operation.to_string()));
    }
    Ok(())
}

fn switch_branch(root: &str, state: &GitState, branch: &str) -> Result<GitOutput, GitError> {
    let info =
        find_branch(state, branch).ok_or_else(|| GitError::BranchNotFound(branch.to_string()))?;

    if info.current {
        return Ok(GitOutput {
            success: true,
            stdout: format!("Ya estás en la rama {branch}\n"),
            stderr: String::new(),
        });
    }

    if info.kind == "local" {
        return run_git(root, &["switch", branch]);
    }

    let local_name = branch
        .split_once('/')
        .map(|(_, name)| name)
        .filter(|name| !name.is_empty())
        .ok_or_else(|| GitError::InvalidBranch(branch.to_string()))?;

    if state
        .branches
        .iter()
        .any(|candidate| candidate.kind == "local" && candidate.name == local_name)
    {
        return run_git(root, &["switch", local_name]);
    }

    run_git(root, &["switch", "-c", local_name, "--track", branch])
}

fn result_for(root: &str, action: &str, output: GitOutput) -> Result<GitOperationResult, GitError> {
    let state = state_for_root(root)?;
    Ok(GitOperationResult {
        success: output.success,
        action: action.to_string(),
        message: output_message(&output),
        stdout: truncate_output(&output.stdout),
        stderr: truncate_output(&output.stderr),
        state,
    })
}

pub fn fetch(path: &str) -> Result<GitOperationResult, GitError> {
    with_write_lock(|| {
        let root = repo_root(path)?;
        let output = run_git(&root, &["fetch", "--all", "--prune"])?;
        result_for(&root, "fetch", output)
    })
}

pub fn pull(path: &str, mode: &str) -> Result<GitOperationResult, GitError> {
    with_write_lock(|| {
        let root = repo_root(path)?;
        let state = state_for_root(&root)?;
        ensure_no_operation(&state)?;
        ensure_clean(&state)?;

        if state.current_branch.is_none() {
            return Err(GitError::CurrentBranch("HEAD desacoplado".to_string()));
        }
        if state.upstream.is_none() {
            return Err(GitError::NoUpstream);
        }

        let output = match mode {
            "merge" => run_git(&root, &["pull", "--no-rebase", "--no-edit"])?,
            "rebase" => run_git(&root, &["pull", "--rebase"])?,
            "ff-only" => run_git(&root, &["pull", "--ff-only"])?,
            _ => {
                return Err(GitError::Command(format!(
                    "Estrategia de pull inválida: {mode}"
                )));
            }
        };

        result_for(&root, "pull", output)
    })
}

pub fn checkout(path: &str, branch: &str) -> Result<GitOperationResult, GitError> {
    with_write_lock(|| {
        let root = repo_root(path)?;
        let state = state_for_root(&root)?;
        ensure_no_operation(&state)?;
        ensure_clean(&state)?;

        let branch = validate_branch(&root, branch)?;
        let output = switch_branch(&root, &state, &branch)?;
        result_for(&root, "checkout", output)
    })
}

pub fn create_branch(
    path: &str,
    name: &str,
    start_point: Option<&str>,
) -> Result<GitOperationResult, GitError> {
    with_write_lock(|| {
        let root = repo_root(path)?;
        let state = state_for_root(&root)?;
        ensure_no_operation(&state)?;

        let name = validate_branch(&root, name)?;
        if find_branch(&state, &name).is_some() {
            return Err(GitError::InvalidBranch(format!("La rama {name} ya existe")));
        }

        let output = if let Some(start) = start_point.filter(|value| !value.trim().is_empty()) {
            let start = validate_branch(&root, start)?;
            let start_info = find_branch(&state, &start)
                .ok_or_else(|| GitError::BranchNotFound(start.clone()))?;
            if start_info.kind == "remote" {
                run_git(&root, &["switch", "-c", &name, "--track", &start])?
            } else {
                run_git(&root, &["switch", "-c", &name, &start])?
            }
        } else {
            run_git(&root, &["switch", "-c", &name])?
        };

        result_for(&root, "create-branch", output)
    })
}

pub fn delete_branch(path: &str, name: &str) -> Result<GitOperationResult, GitError> {
    with_write_lock(|| {
        let root = repo_root(path)?;
        let state = state_for_root(&root)?;
        ensure_no_operation(&state)?;

        let name = validate_branch(&root, name)?;
        if state.current_branch.as_deref() == Some(name.as_str()) {
            return Err(GitError::CurrentBranch(name));
        }
        let info =
            find_branch(&state, &name).ok_or_else(|| GitError::BranchNotFound(name.clone()))?;
        if info.kind != "local" {
            return Err(GitError::InvalidBranch(format!(
                "No se pueden eliminar ramas remotas: {name}"
            )));
        }

        let output = run_git(&root, &["branch", "-d", &name])?;
        result_for(&root, "delete-branch", output)
    })
}

pub fn merge(path: &str, source: &str, target: &str) -> Result<GitOperationResult, GitError> {
    with_write_lock(|| {
        let root = repo_root(path)?;
        let state = state_for_root(&root)?;
        ensure_no_operation(&state)?;
        ensure_clean(&state)?;

        let source = validate_branch(&root, source)?;
        let target = validate_branch(&root, target)?;

        let source_info =
            find_branch(&state, &source).ok_or_else(|| GitError::BranchNotFound(source.clone()))?;
        let target_info =
            find_branch(&state, &target).ok_or_else(|| GitError::BranchNotFound(target.clone()))?;

        if target_info.kind != "local" {
            return Err(GitError::InvalidBranch(format!(
                "La rama destino debe ser local: {target}"
            )));
        }
        if source == target {
            return Err(GitError::InvalidBranch(
                "La rama origen y destino son la misma".to_string(),
            ));
        }
        if source_info.name == target_info.name {
            return Err(GitError::InvalidBranch(
                "La rama origen y destino son la misma".to_string(),
            ));
        }

        if state.current_branch.as_deref() != Some(target.as_str()) {
            let switched = switch_branch(&root, &state, &target)?;
            if !switched.success {
                return result_for(&root, "merge", switched);
            }
        }

        let output = run_git(&root, &["merge", "--no-edit", &source])?;
        result_for(&root, "merge", output)
    })
}

pub fn abort_operation(path: &str) -> Result<GitOperationResult, GitError> {
    with_write_lock(|| {
        let root = repo_root(path)?;
        let state = state_for_root(&root)?;
        let operation = state
            .operation
            .as_deref()
            .ok_or_else(|| GitError::Command("No hay una operación Git en curso".to_string()))?;

        let args: &[&str] = match operation {
            "merge" => &["merge", "--abort"],
            "rebase" => &["rebase", "--abort"],
            "cherry-pick" => &["cherry-pick", "--abort"],
            "revert" => &["revert", "--abort"],
            other => {
                return Err(GitError::Command(format!(
                    "No se puede abortar la operación: {other}"
                )));
            }
        };

        let output = run_git(&root, args)?;
        result_for(&root, "abort", output)
    })
}

pub fn push(path: &str, remote: Option<String>) -> Result<GitOperationResult, GitError> {
    with_write_lock(|| {
        let root = repo_root(path)?;
        let state = state_for_root(&root)?;
        ensure_no_operation(&state)?;

        let branch = state
            .current_branch
            .clone()
            .ok_or_else(|| GitError::CurrentBranch("HEAD desacoplado".to_string()))?;
        if state.remotes.is_empty() {
            return Err(GitError::NoRemote);
        }

        let remote = remote
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        if let Some(value) = remote.as_deref() {
            if !state.remotes.iter().any(|candidate| candidate == value) {
                return Err(GitError::Command(format!("Remoto no encontrado: {value}")));
            }
        }

        let upstream_remote = state
            .upstream
            .as_deref()
            .and_then(|value| value.split_once('/'))
            .map(|(name, _)| name.to_string());
        let selected_remote = remote
            .clone()
            .or(upstream_remote)
            .or_else(|| {
                state
                    .remotes
                    .iter()
                    .find(|value| value.as_str() == "origin")
                    .cloned()
            })
            .or_else(|| state.remotes.first().cloned())
            .ok_or(GitError::NoRemote)?;

        let output = if state.upstream.is_some() && remote.is_none() {
            run_git(&root, &["push"])?
        } else if state.upstream.is_none() {
            run_git(
                &root,
                &["push", "--set-upstream", &selected_remote, &branch],
            )?
        } else {
            run_git(&root, &["push", &selected_remote, &branch])?
        };

        result_for(&root, "push", output)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn init_repo() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cardumen-deck-git-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.to_string_lossy().to_string();
        run_git(&path, &["init", "-b", "main"]).unwrap();
        run_git(&path, &["config", "user.email", "test@example.com"]).unwrap();
        run_git(&path, &["config", "user.name", "Test User"]).unwrap();
        std::fs::write(dir.join("file.txt"), "base\n").unwrap();
        run_git(&path, &["add", "file.txt"]).unwrap();
        run_git(&path, &["commit", "-m", "initial"]).unwrap();
        dir
    }

    fn write_and_commit(path: &Path, content: &str, message: &str) {
        let path_str = path.to_string_lossy().to_string();
        std::fs::write(path.join("file.txt"), content).unwrap();
        run_git(&path_str, &["add", "file.txt"]).unwrap();
        run_git(&path_str, &["commit", "-m", message]).unwrap();
    }

    #[test]
    fn non_git_directory_has_no_branch() {
        let dir = std::env::temp_dir().join(format!("cardumen-deck-git-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        let branches = branches_for(&[dir.to_string_lossy().to_string()]);
        assert!(branches.is_empty());

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn reports_clean_repository_state() {
        let dir = init_repo();
        let path = dir.to_string_lossy().to_string();

        let state = state_for(&path).expect("state");
        assert_eq!(state.current_branch.as_deref(), Some("main"));
        assert!(!state.dirty);
        assert!(
            state
                .branches
                .iter()
                .any(|branch| branch.name == "main" && branch.current)
        );

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn detects_dirty_tree_and_blocks_checkout() {
        let dir = init_repo();
        let path = dir.to_string_lossy().to_string();
        std::fs::write(dir.join("file.txt"), "changed\n").unwrap();

        let state = state_for(&path).expect("state");
        assert!(state.dirty);
        assert!(state.changed_files.iter().any(|file| file == "file.txt"));

        let error = checkout(&path, "main").expect_err("checkout should fail");
        assert!(matches!(error, GitError::DirtyWorkTree(_)));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn untracked_files_do_not_block_checkout() {
        let dir = init_repo();
        let path = dir.to_string_lossy().to_string();
        std::fs::write(dir.join("untracked.txt"), "new\n").unwrap();

        let state = state_for(&path).expect("state");
        assert!(!state.dirty);
        assert!(
            state
                .untracked_files
                .iter()
                .any(|file| file == "untracked.txt")
        );

        let result = checkout(&path, "main").expect("checkout should work");
        assert!(result.success);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn creates_and_switches_branches() {
        let dir = init_repo();
        let path = dir.to_string_lossy().to_string();

        let created = create_branch(&path, "feature/test", None).expect("create branch");
        assert!(created.success);
        assert_eq!(
            created.state.current_branch.as_deref(),
            Some("feature/test")
        );

        let switched = checkout(&path, "main").expect("checkout main");
        assert!(switched.success);
        assert_eq!(switched.state.current_branch.as_deref(), Some("main"));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn merges_source_into_target() {
        let dir = init_repo();
        let path = dir.to_string_lossy().to_string();

        run_git(&path, &["switch", "-c", "feature"]).unwrap();
        write_and_commit(&dir, "feature\n", "feature change");
        run_git(&path, &["switch", "main"]).unwrap();

        let result = merge(&path, "feature", "main").expect("merge");
        assert!(result.success);
        assert_eq!(result.state.current_branch.as_deref(), Some("main"));
        assert_eq!(
            std::fs::read_to_string(dir.join("file.txt"))
                .unwrap()
                .trim(),
            "feature"
        );

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn reports_conflict_and_aborts_merge() {
        let dir = init_repo();
        let path = dir.to_string_lossy().to_string();

        run_git(&path, &["switch", "-c", "feature"]).unwrap();
        write_and_commit(&dir, "feature\n", "feature change");
        run_git(&path, &["switch", "main"]).unwrap();
        write_and_commit(&dir, "main\n", "main change");

        let result = merge(&path, "feature", "main").expect("merge result");
        assert!(!result.success);
        assert_eq!(result.state.operation.as_deref(), Some("merge"));
        assert!(
            result
                .state
                .conflict_files
                .iter()
                .any(|file| file == "file.txt")
        );

        let aborted = abort_operation(&path).expect("abort merge");
        assert!(aborted.success);
        assert!(aborted.state.operation.is_none());

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn rejects_invalid_branch_names() {
        let dir = init_repo();
        let path = dir.to_string_lossy().to_string();

        let error = checkout(&path, "-bad").expect_err("invalid branch");
        assert!(matches!(error, GitError::InvalidBranch(_)));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn deduplicates_repositories_by_root() {
        let dir = init_repo();
        let subdir = dir.join("sub");
        std::fs::create_dir_all(&subdir).unwrap();

        let repositories = repositories_for(&[
            dir.to_string_lossy().to_string(),
            subdir.to_string_lossy().to_string(),
        ]);
        assert_eq!(repositories.len(), 1);

        let _ = std::fs::remove_dir_all(dir);
    }
}
