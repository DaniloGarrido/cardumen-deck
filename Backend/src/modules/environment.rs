use std::collections::HashMap;
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use once_cell::sync::Lazy;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::modules::config::setup_logs_dir;
use crate::modules::logs::Tailer;
use crate::modules::process::{detect_venv, resolve_cwd};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentInfo {
    pub stack: String,
    pub display_name: String,
    pub setup_command: Option<String>,
    pub run_command: Option<String>,
    pub suggested_preset: Option<String>,
    pub suggested_port: Option<u16>,
    pub venv_path: Option<String>,
    pub package_manager: Option<String>,
    pub detected_files: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupStatus {
    pub task_id: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub log_path: String,
    pub started_at: u64,
    pub ended_at: Option<u64>,
}

#[derive(Debug, Clone)]
struct SetupTask {
    pid: u32,
    status: String,
    exit_code: Option<i32>,
    log_path: PathBuf,
    started_at: u64,
    ended_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupLogEvent {
    pub task_id: String,
    pub lines: Vec<String>,
}

static SETUP_TASKS: Lazy<Arc<Mutex<HashMap<String, SetupTask>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));
static SETUP_LOG_STREAMS: Lazy<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn quote_path(path: &Path) -> String {
    let value = path.to_string_lossy();
    if value.contains(' ') {
        format!("\"{}\"", value)
    } else {
        value.to_string()
    }
}

fn push_file(files: &mut Vec<String>, cwd: &Path, name: &str) {
    if cwd.join(name).exists() {
        files.push(name.to_string());
    }
}

fn find_file_with_extension(cwd: &Path, extension: &str) -> Option<String> {
    std::fs::read_dir(cwd)
        .ok()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .find(|name| name.ends_with(extension))
}

fn command_available(name: &str) -> bool {
    which::which(name).is_ok()
}

fn node_package_manager(cwd: &Path) -> &'static str {
    if cwd.join("pnpm-lock.yaml").exists() {
        "pnpm"
    } else if cwd.join("yarn.lock").exists() {
        "yarn"
    } else if cwd.join("bun.lockb").exists() || cwd.join("bun.lock").exists() {
        "bun"
    } else {
        "npm"
    }
}

fn node_run_command(cwd: &Path, package_manager: &str) -> String {
    let package_json = std::fs::read_to_string(cwd.join("package.json")).ok();
    if let Some(content) = package_json {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(scripts) = json.get("scripts").and_then(|value| value.as_object()) {
                if scripts.contains_key("dev") {
                    return format!("{} run dev", package_manager);
                }
                if scripts.contains_key("start") {
                    return format!("{} run start", package_manager);
                }
            }
        }
    }

    format!("{} run dev", package_manager)
}

fn python_run_command(cwd: &Path, prefix: Option<&str>) -> String {
    let runner = if cwd.join("manage.py").exists() {
        "python manage.py runserver"
    } else if cwd.join("app.py").exists() {
        "python app.py"
    } else {
        "python main.py"
    };

    match prefix {
        Some(prefix) => format!("{} {}", prefix, runner),
        None => runner.to_string(),
    }
}

fn detected_python_files(cwd: &Path) -> Vec<String> {
    let mut files = Vec::new();
    for name in ["manage.py", "requirements.txt", "pyproject.toml", "Pipfile"] {
        push_file(&mut files, cwd, name);
    }
    files
}

fn detect_python(cwd: &Path) -> Option<EnvironmentInfo> {
    let files = detected_python_files(cwd);
    if files.is_empty() {
        return None;
    }

    let mut warnings = Vec::new();
    let venv = detect_venv(cwd);
    let has_requirements = cwd.join("requirements.txt").is_file();
    let has_pyproject = cwd.join("pyproject.toml").is_file();
    let has_pipfile = cwd.join("Pipfile").is_file();
    let poetry_project = has_pyproject
        && std::fs::read_to_string(cwd.join("pyproject.toml"))
            .map(|content| content.contains("[tool.poetry]"))
            .unwrap_or(false);

    let (setup_command, run_command, package_manager, venv_path) =
        if has_pipfile && command_available("pipenv") {
            (
                Some("pipenv install".to_string()),
                Some(python_run_command(cwd, Some("pipenv run"))),
                Some("pipenv".to_string()),
                None,
            )
        } else if poetry_project && command_available("poetry") {
            (
                Some("poetry install".to_string()),
                Some(python_run_command(cwd, Some("poetry run"))),
                Some("poetry".to_string()),
                None,
            )
        } else {
            if has_pipfile {
                warnings.push("Pipfile detectado, pero pipenv no está instalado".to_string());
            }
            if poetry_project && !command_available("poetry") {
                warnings.push(
                    "pyproject.toml de Poetry detectado, pero poetry no está instalado".to_string(),
                );
            }

            let target_venv = venv.clone().unwrap_or_else(|| cwd.join(".venv"));
            let python = target_venv.join("Scripts").join("python.exe");
            let mut setup_parts = Vec::new();

            if venv.is_none() {
                if command_available("python") {
                    setup_parts.push("python -m venv .venv".to_string());
                } else {
                    warnings.push("No se encontró python en PATH".to_string());
                    setup_parts.push("python -m venv .venv".to_string());
                }
            }

            setup_parts.push(format!(
                "{} -m pip install --upgrade pip",
                quote_path(&python)
            ));
            if has_requirements {
                setup_parts.push(format!(
                    "{} -m pip install -r requirements.txt",
                    quote_path(&python)
                ));
            } else if has_pyproject {
                setup_parts.push(format!("{} -m pip install -e .", quote_path(&python)));
            }

            (
                Some(setup_parts.join(" && ")),
                Some(python_run_command(cwd, None)),
                Some("pip".to_string()),
                venv.map(|path| path.to_string_lossy().to_string()),
            )
        };

    Some(EnvironmentInfo {
        stack: "python".to_string(),
        display_name: if cwd.join("manage.py").is_file() {
            "Python / Django".to_string()
        } else {
            "Python".to_string()
        },
        setup_command,
        run_command,
        suggested_preset: Some("python".to_string()),
        suggested_port: Some(8000),
        venv_path,
        package_manager,
        detected_files: files,
        warnings,
    })
}

fn detect_node(cwd: &Path) -> Option<EnvironmentInfo> {
    if !cwd.join("package.json").is_file() {
        return None;
    }

    let mut files = Vec::new();
    for name in [
        "package.json",
        "package-lock.json",
        "pnpm-lock.yaml",
        "yarn.lock",
        "bun.lockb",
        "bun.lock",
    ] {
        push_file(&mut files, cwd, name);
    }

    let package_manager = node_package_manager(cwd);
    let mut warnings = Vec::new();
    if !command_available(package_manager) {
        warnings.push(format!("No se encontró {} en PATH", package_manager));
    }

    let package_json = std::fs::read_to_string(cwd.join("package.json")).unwrap_or_default();
    let display_name = if package_json.contains("\"vue\"") || package_json.contains("\"vite\"") {
        "Node / Vue"
    } else {
        "Node"
    };

    Some(EnvironmentInfo {
        stack: "node".to_string(),
        display_name: display_name.to_string(),
        setup_command: Some(format!("{} install", package_manager)),
        run_command: Some(node_run_command(cwd, package_manager)),
        suggested_preset: Some("vue".to_string()),
        suggested_port: Some(5173),
        venv_path: None,
        package_manager: Some(package_manager.to_string()),
        detected_files: files,
        warnings,
    })
}

fn detect_spring_boot(cwd: &Path) -> Option<EnvironmentInfo> {
    let has_maven = cwd.join("pom.xml").is_file();
    let has_gradle = cwd.join("build.gradle").is_file() || cwd.join("build.gradle.kts").is_file();
    if !has_maven && !has_gradle {
        return None;
    }

    let mut files = Vec::new();
    for name in [
        "pom.xml",
        "build.gradle",
        "build.gradle.kts",
        "mvnw.cmd",
        "gradlew.bat",
    ] {
        push_file(&mut files, cwd, name);
    }

    let mut warnings = Vec::new();
    let (setup_command, run_command, package_manager) = if has_maven {
        if cwd.join("mvnw.cmd").is_file() {
            (
                "mvnw.cmd -q -DskipTests dependency:go-offline".to_string(),
                "mvnw.cmd spring-boot:run".to_string(),
                "maven".to_string(),
            )
        } else {
            if !command_available("mvn") {
                warnings.push("No se encontró mvnw.cmd ni mvn en PATH".to_string());
            }
            (
                "mvn -q -DskipTests dependency:go-offline".to_string(),
                "mvn spring-boot:run".to_string(),
                "maven".to_string(),
            )
        }
    } else if cwd.join("gradlew.bat").is_file() {
        (
            "gradlew.bat -q classes".to_string(),
            "gradlew.bat bootRun".to_string(),
            "gradle".to_string(),
        )
    } else {
        if !command_available("gradle") {
            warnings.push("No se encontró gradlew.bat ni gradle en PATH".to_string());
        }
        (
            "gradle -q classes".to_string(),
            "gradle bootRun".to_string(),
            "gradle".to_string(),
        )
    };

    Some(EnvironmentInfo {
        stack: "spring-boot".to_string(),
        display_name: "Spring Boot".to_string(),
        setup_command: Some(setup_command),
        run_command: Some(run_command),
        suggested_preset: Some("spring-boot".to_string()),
        suggested_port: Some(8080),
        venv_path: None,
        package_manager: Some(package_manager),
        detected_files: files,
        warnings,
    })
}

#[allow(clippy::too_many_arguments)]
fn detect_simple_stack(
    file_name: &str,
    stack: &str,
    display_name: &str,
    setup_command: &str,
    run_command: &str,
    suggested_preset: &str,
    suggested_port: u16,
    tool: &str,
) -> EnvironmentInfo {
    let mut warnings = Vec::new();
    if !command_available(tool) {
        warnings.push(format!("No se encontró {} en PATH", tool));
    }

    EnvironmentInfo {
        stack: stack.to_string(),
        display_name: display_name.to_string(),
        setup_command: Some(setup_command.to_string()),
        run_command: Some(run_command.to_string()),
        suggested_preset: Some(suggested_preset.to_string()),
        suggested_port: Some(suggested_port),
        venv_path: None,
        package_manager: None,
        detected_files: vec![file_name.to_string()],
        warnings,
    }
}

pub fn detect_environment(cwd: &str) -> Result<EnvironmentInfo, String> {
    let path = resolve_cwd(cwd);
    if !path.is_dir() {
        return Err(format!("Carpeta no encontrada: {}", path.display()));
    }

    if let Some(info) = detect_python(&path) {
        return Ok(info);
    }
    if let Some(info) = detect_spring_boot(&path) {
        return Ok(info);
    }
    if let Some(info) = detect_node(&path) {
        return Ok(info);
    }
    if let Some(file_name) = find_file_with_extension(&path, ".csproj")
        .or_else(|| find_file_with_extension(&path, ".sln"))
    {
        return Ok(detect_simple_stack(
            &file_name,
            "dotnet",
            ".NET",
            "dotnet restore",
            "dotnet run",
            "dotnet",
            5000,
            "dotnet",
        ));
    }
    if path.join("Cargo.toml").is_file() {
        return Ok(detect_simple_stack(
            "Cargo.toml",
            "rust",
            "Rust",
            "cargo fetch",
            "cargo run",
            "rust",
            3000,
            "cargo",
        ));
    }
    if path.join("go.mod").is_file() {
        return Ok(detect_simple_stack(
            "go.mod",
            "go",
            "Go",
            "go mod download",
            "go run .",
            "go",
            3000,
            "go",
        ));
    }

    Ok(EnvironmentInfo {
        stack: "custom".to_string(),
        display_name: "Personalizado".to_string(),
        setup_command: None,
        run_command: None,
        suggested_preset: Some("custom".to_string()),
        suggested_port: None,
        venv_path: None,
        package_manager: None,
        detected_files: Vec::new(),
        warnings: Vec::new(),
    })
}

fn task_status(task_id: &str, task: &SetupTask) -> SetupStatus {
    SetupStatus {
        task_id: task_id.to_string(),
        status: task.status.clone(),
        exit_code: task.exit_code,
        log_path: task.log_path.to_string_lossy().to_string(),
        started_at: task.started_at,
        ended_at: task.ended_at,
    }
}

pub fn start_setup(cwd: &str, command: &str) -> Result<SetupStatus, String> {
    let resolved_cwd = resolve_cwd(cwd);
    if !resolved_cwd.is_dir() {
        return Err(format!("Carpeta no encontrada: {}", resolved_cwd.display()));
    }

    let command = command.trim();
    if command.is_empty() {
        return Err("El comando de preparación está vacío".to_string());
    }

    let task_id = Uuid::new_v4().to_string();
    let log_dir = setup_logs_dir().map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&log_dir).map_err(|error| error.to_string())?;
    let log_path = log_dir.join(format!("{}.log", task_id));
    let mut log_file = std::fs::File::create(&log_path).map_err(|error| error.to_string())?;
    writeln!(log_file, "[Cardumen Deck] Preparando entorno: {}", command)
        .map_err(|error| error.to_string())?;
    writeln!(log_file, "[Cardumen Deck] cwd: {}", resolved_cwd.display())
        .map_err(|error| error.to_string())?;
    log_file.flush().map_err(|error| error.to_string())?;
    let log_file_stderr = log_file.try_clone().map_err(|error| error.to_string())?;

    let mut cmd = Command::new("cmd");
    cmd.arg("/C");
    cmd.arg(command);
    cmd.current_dir(&resolved_cwd);
    cmd.env("PYTHONUNBUFFERED", "1");
    cmd.env("PYTHONIOENCODING", "utf-8");
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::from(log_file));
    cmd.stderr(Stdio::from(log_file_stderr));
    cmd.creation_flags(0x08000000);

    let mut child = cmd.spawn().map_err(|error| error.to_string())?;
    let task = SetupTask {
        pid: child.id(),
        status: "running".to_string(),
        exit_code: None,
        log_path: log_path.clone(),
        started_at: now_seconds(),
        ended_at: None,
    };

    SETUP_TASKS
        .lock()
        .unwrap()
        .insert(task_id.clone(), task.clone());

    let thread_task_id = task_id.clone();
    let thread_log_path = log_path.clone();
    std::thread::spawn(move || {
        let exit_code = child.wait().ok().and_then(|status| status.code());
        {
            let mut tasks = SETUP_TASKS.lock().unwrap();
            if let Some(task) = tasks.get_mut(&thread_task_id) {
                if task.status == "running" {
                    task.status = if exit_code == Some(0) {
                        "success".to_string()
                    } else {
                        "error".to_string()
                    };
                    task.exit_code = exit_code;
                    task.ended_at = Some(now_seconds());
                }
            }
        }

        if let Ok(mut log_file) = std::fs::OpenOptions::new()
            .append(true)
            .open(&thread_log_path)
        {
            let message = match exit_code {
                Some(0) => {
                    "[Cardumen Deck] Preparación completada correctamente (código 0)".to_string()
                }
                Some(code) => format!("[Cardumen Deck] Preparación finalizada con código {}", code),
                None => "[Cardumen Deck] Preparación finalizada sin código de salida".to_string(),
            };
            let _ = writeln!(log_file, "\n{}", message);
            let _ = log_file.flush();
        }
    });

    Ok(task_status(&task_id, &task))
}

pub fn get_setup_status(task_id: &str) -> Result<SetupStatus, String> {
    let tasks = SETUP_TASKS.lock().unwrap();
    tasks
        .get(task_id)
        .map(|task| task_status(task_id, task))
        .ok_or_else(|| "Preparación no encontrada".to_string())
}

pub fn read_setup_log(
    task_id: &str,
    lines: Option<usize>,
) -> Result<crate::modules::config::LogOutput, String> {
    let log_path = {
        let tasks = SETUP_TASKS.lock().unwrap();
        tasks
            .get(task_id)
            .map(|task| task.log_path.clone())
            .ok_or_else(|| "Preparación no encontrada".to_string())?
    };

    crate::modules::logs::read_log_tail(&log_path, task_id, lines)
        .map_err(|error| error.to_string())
}

pub fn cancel_setup(task_id: &str) -> Result<(), String> {
    let mut tasks = SETUP_TASKS.lock().unwrap();
    let task = tasks
        .get_mut(task_id)
        .ok_or_else(|| "Preparación no encontrada".to_string())?;

    if task.status == "running" {
        let _ = Command::new("taskkill")
            .args(["/PID", &task.pid.to_string(), "/T", "/F"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        task.status = "cancelled".to_string();
        task.ended_at = Some(now_seconds());
    }

    Ok(())
}

pub fn start_setup_log_stream(app: AppHandle, task_id: String) -> Result<(), String> {
    stop_setup_log_stream(&task_id);

    let log_path = {
        let tasks = SETUP_TASKS.lock().unwrap();
        tasks
            .get(&task_id)
            .map(|task| task.log_path.clone())
            .ok_or_else(|| "Preparación no encontrada".to_string())?
    };

    let cancel = Arc::new(AtomicBool::new(false));
    SETUP_LOG_STREAMS
        .lock()
        .unwrap()
        .insert(task_id.clone(), cancel.clone());

    std::thread::spawn(move || {
        let mut tailer = Tailer::open(&log_path, true).ok();

        loop {
            if cancel.load(Ordering::Relaxed) {
                break;
            }

            if tailer.is_none() {
                tailer = Tailer::open(&log_path, false).ok();
            }

            if let Some(current_tailer) = tailer.as_mut() {
                match current_tailer.poll() {
                    Ok(lines) if !lines.is_empty() => {
                        let _ = app.emit(
                            "setup-log",
                            SetupLogEvent {
                                task_id: task_id.clone(),
                                lines,
                            },
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        eprintln!("Error leyendo stream de preparación: {error}");
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    });

    Ok(())
}

pub fn stop_setup_log_stream(task_id: &str) {
    if let Some(cancel) = SETUP_LOG_STREAMS.lock().unwrap().remove(task_id) {
        cancel.store(true, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn setup_runner_captures_output_and_success() {
        let dir = std::env::temp_dir().join(format!("cardumen-deck-setup-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();

        let started = start_setup(dir.to_str().unwrap(), "echo setup-ok").unwrap();
        let mut status = started.clone();
        for _ in 0..50 {
            if status.status != "running" {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
            status = get_setup_status(&started.task_id).unwrap();
        }

        assert_eq!(status.status, "success");
        let output = read_setup_log(&started.task_id, Some(20)).unwrap();
        assert!(output.lines.iter().any(|line| line.contains("setup-ok")));

        let _ = std::fs::remove_file(&status.log_path);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn detects_python_venv_and_setup_command() {
        let dir = std::env::temp_dir().join(format!("cardumen-deck-env-{}", Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("venv").join("Scripts")).unwrap();
        std::fs::write(dir.join("venv").join("Scripts").join("python.exe"), b"").unwrap();
        std::fs::write(dir.join("venv").join("pyvenv.cfg"), b"").unwrap();
        std::fs::write(dir.join("manage.py"), b"").unwrap();
        std::fs::write(dir.join("requirements.txt"), b"").unwrap();

        let info = detect_environment(dir.to_str().unwrap()).unwrap();
        assert_eq!(info.stack, "python");
        assert!(info.venv_path.is_some());
        assert!(
            info.setup_command
                .as_deref()
                .unwrap()
                .contains("pip install -r requirements.txt")
        );

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn detects_dotnet_rust_and_go_presets() {
        let cases = [
            ("dotnet", "app.csproj", "dotnet"),
            ("rust", "Cargo.toml", "rust"),
            ("go", "go.mod", "go"),
        ];

        for (folder, file, expected_preset) in cases {
            let dir =
                std::env::temp_dir().join(format!("cardumen-deck-{}-{}", folder, Uuid::new_v4()));
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join(file), b"").unwrap();

            let info = detect_environment(dir.to_str().unwrap()).unwrap();
            assert_eq!(info.stack, folder);
            assert_eq!(info.suggested_preset.as_deref(), Some(expected_preset));

            let _ = std::fs::remove_dir_all(dir);
        }
    }
}
