use std::collections::HashMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use once_cell::sync::Lazy;
use serde::Serialize;
use strip_ansi_escapes::strip;
use tauri::{AppHandle, Emitter};

use crate::modules::config::{LogOutput, RunInfo, service_log_dir};

#[derive(Debug, thiserror::Error)]
pub enum LogError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Config error: {0}")]
    Config(#[from] crate::modules::config::ConfigError),
    #[error("Run not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, LogError>;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceLogEvent {
    pub service_id: String,
    pub run_id: String,
    pub lines: Vec<String>,
}

static SERVICE_LOG_STREAMS: Lazy<Mutex<HashMap<String, Arc<AtomicBool>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub struct Tailer {
    file: fs::File,
    position: u64,
    buffer: String,
}

impl Tailer {
    pub fn open(path: &Path, start_at_end: bool) -> std::io::Result<Self> {
        let mut file = fs::File::open(path)?;
        let position = if start_at_end {
            file.metadata()?.len()
        } else {
            0
        };
        file.seek(SeekFrom::Start(position))?;
        Ok(Self {
            file,
            position,
            buffer: String::new(),
        })
    }

    pub fn poll(&mut self) -> std::io::Result<Vec<String>> {
        let len = self.file.metadata()?.len();
        if len < self.position {
            self.position = 0;
            self.buffer.clear();
        }
        if len == self.position {
            return Ok(Vec::new());
        }

        self.file.seek(SeekFrom::Start(self.position))?;
        let mut bytes = Vec::new();
        self.file
            .by_ref()
            .take(len - self.position)
            .read_to_end(&mut bytes)?;
        self.position = len;
        self.buffer.push_str(&String::from_utf8_lossy(&bytes));

        let mut lines = Vec::new();
        while let Some(index) = self.buffer.find('\n') {
            let line = self.buffer[..index].trim_end_matches('\r').to_string();
            self.buffer.drain(..=index);
            lines.push(String::from_utf8_lossy(&strip(line)).to_string());
        }

        Ok(lines)
    }
}

const FATAL_PATTERNS: &[&str] = &[
    "modulenotfounderror",
    "importerror",
    "syntaxerror",
    "exception in thread",
    "npm err!",
    "cannot find module",
    "eaddrinuse",
    "address already in use",
    "failed to compile",
    "build failure",
    "application failed to start",
    "is not recognized as an internal or external command",
    "unhandled exception",
    "fatal error",
    "traceback (most recent call last)",
];

fn entry_modified(entry: &fs::DirEntry) -> SystemTime {
    entry
        .metadata()
        .and_then(|metadata| metadata.modified())
        .unwrap_or(UNIX_EPOCH)
}

fn log_entries(service_id: &str) -> Result<Vec<fs::DirEntry>> {
    let log_dir = service_log_dir(service_id)?;
    if !log_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<_> = fs::read_dir(&log_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "log"))
        .collect();
    entries.sort_by_key(|entry| std::cmp::Reverse(entry_modified(entry)));
    Ok(entries)
}

pub fn latest_log_path(service_id: &str, run_id: Option<&str>) -> Result<PathBuf> {
    if let Some(run_id) = run_id {
        let path = service_log_dir(service_id)?.join(format!("{}.log", run_id));
        if path.exists() {
            return Ok(path);
        }
        return Err(LogError::NotFound(run_id.to_string()));
    }

    let entries = log_entries(service_id)?;
    entries
        .first()
        .map(|entry| entry.path())
        .ok_or_else(|| LogError::NotFound(service_id.to_string()))
}

fn read_tail_bytes(path: &Path, max_bytes: u64) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let len = file.metadata()?.len();
    let start = len.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start))?;

    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    if start > 0 {
        if let Some(first_newline) = buffer.find('\n') {
            buffer = buffer[first_newline + 1..].to_string();
        }
    }

    Ok(buffer)
}

pub fn read_log_tail(path: &Path, run_id: &str, lines: Option<usize>) -> Result<LogOutput> {
    let content = fs::read_to_string(path)?;
    let mut lines_vec: Vec<String> = content.lines().map(|line| line.to_string()).collect();
    let total_lines = lines_vec.len();

    let limit = lines.unwrap_or(500);
    let truncated = total_lines > limit;
    if truncated {
        lines_vec = lines_vec.split_off(total_lines - limit);
    }

    let lines: Vec<String> = lines_vec
        .into_iter()
        .map(|line| String::from_utf8_lossy(&strip(line)).to_string())
        .collect();

    let updated_at = fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs());

    Ok(LogOutput {
        run_id: run_id.to_string(),
        lines,
        truncated,
        total_lines,
        updated_at,
    })
}

pub async fn logs_read_logs(
    service_id: &str,
    run_id: Option<String>,
    lines: Option<usize>,
) -> Result<LogOutput> {
    let path = latest_log_path(service_id, run_id.as_deref())?;
    let resolved_run_id = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(service_id);
    read_log_tail(&path, resolved_run_id, lines)
}

pub fn detect_fatal_error(service_id: &str, run_id: Option<&str>) -> Option<String> {
    let path = latest_log_path(service_id, run_id)
        .or_else(|_| latest_log_path(service_id, None))
        .ok()?;
    let content = read_tail_bytes(&path, 128 * 1024).ok()?;

    for line in content.lines().rev() {
        let stripped = String::from_utf8_lossy(&strip(line)).to_string();
        let normalized = stripped.to_lowercase();
        if FATAL_PATTERNS
            .iter()
            .any(|pattern| normalized.contains(pattern))
        {
            let trimmed = stripped.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.chars().take(300).collect());
            }
        }
    }

    None
}

pub fn start_log_stream(
    app: AppHandle,
    service_id: String,
    run_id: Option<String>,
) -> std::result::Result<(), String> {
    stop_log_stream(&service_id);

    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut streams = SERVICE_LOG_STREAMS.lock().unwrap();
        streams.insert(service_id.clone(), cancel.clone());
    }

    let initial_path = if let Some(run_id) = run_id.as_deref() {
        latest_log_path(&service_id, Some(run_id)).ok()
    } else {
        latest_log_path(&service_id, None).ok()
    };
    let first_open = true;
    let first_start_at_end = initial_path.is_some();

    std::thread::spawn(move || {
        let mut tailer: Option<Tailer> = None;
        let mut tailer_path: Option<PathBuf> = None;
        let mut active_run_id = run_id.clone().unwrap_or_default();
        let mut is_first_open = first_open;

        loop {
            if cancel.load(Ordering::Relaxed) {
                break;
            }

            let target = if let Some(run_id) = run_id.as_deref() {
                latest_log_path(&service_id, Some(run_id)).ok()
            } else {
                latest_log_path(&service_id, None).ok()
            };

            if let Some(path) = target {
                if tailer_path.as_deref() != Some(path.as_path()) {
                    let start_at_end = is_first_open && first_start_at_end;
                    if let Ok(next_tailer) = Tailer::open(&path, start_at_end) {
                        active_run_id = path
                            .file_stem()
                            .and_then(|stem| stem.to_str())
                            .unwrap_or_default()
                            .to_string();
                        tailer = Some(next_tailer);
                        tailer_path = Some(path);
                        is_first_open = false;
                    }
                }
            }

            if let Some(current_tailer) = tailer.as_mut() {
                match current_tailer.poll() {
                    Ok(lines) if !lines.is_empty() => {
                        let _ = app.emit(
                            "service-log",
                            ServiceLogEvent {
                                service_id: service_id.clone(),
                                run_id: active_run_id.clone(),
                                lines,
                            },
                        );
                    }
                    Ok(_) => {}
                    Err(error) => {
                        eprintln!("Error leyendo stream de logs: {error}");
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(200));
        }
    });

    Ok(())
}

pub fn stop_log_stream(service_id: &str) {
    if let Some(cancel) = SERVICE_LOG_STREAMS.lock().unwrap().remove(service_id) {
        cancel.store(true, Ordering::Relaxed);
    }
}

pub async fn list_runs(service_id: &str) -> Result<Vec<RunInfo>> {
    let entries = log_entries(service_id)?;
    let mut runs = Vec::new();

    for entry in entries {
        let path = entry.path();
        if let Some(stem) = path.file_stem() {
            let metadata = entry.metadata()?;
            let start_time = metadata
                .created()
                .unwrap_or(UNIX_EPOCH)
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            runs.push(RunInfo {
                run_id: stem.to_string_lossy().to_string(),
                start_time: chrono::DateTime::from_timestamp(start_time as i64, 0)
                    .unwrap_or_else(chrono::Utc::now)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                end_time: None,
                status: "stopped".to_string(),
                log_path: path.to_string_lossy().to_string(),
            });
        }
    }

    Ok(runs)
}

pub async fn logs_open_log_folder(service_id: &str) -> Result<()> {
    let log_dir = service_log_dir(service_id)?;
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir)?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(log_dir.to_string_lossy().to_string())
            .spawn()?;
    }

    Ok(())
}

pub async fn cleanup_old_logs(
    service_id: &str,
    retention_runs: usize,
    retention_days: u64,
) -> Result<()> {
    let entries = log_entries(service_id)?;
    if entries.is_empty() {
        return Ok(());
    }

    let now = SystemTime::now();
    let cutoff = now - Duration::from_secs(retention_days * 86400);

    for (idx, entry) in entries.iter().enumerate() {
        let should_delete = idx >= retention_runs;
        let modified = entry_modified(entry);
        let older_than_retention = modified < cutoff;

        if should_delete || older_than_retention {
            let _ = fs::remove_file(entry.path());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use uuid::Uuid;

    #[test]
    fn read_tail_returns_last_lines() {
        let service_id = format!("test-{}", Uuid::new_v4());
        let dir = service_log_dir(&service_id).unwrap();
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("run.log");
        let mut file = fs::File::create(&path).unwrap();
        for index in 1..=10 {
            writeln!(file, "line {}", index).unwrap();
        }

        let output = read_log_tail(&path, "run", Some(3)).unwrap();
        assert_eq!(output.lines, vec!["line 8", "line 9", "line 10"]);
        assert!(output.truncated);
        assert_eq!(output.total_lines, 10);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn tailer_reads_only_new_lines() {
        let dir = std::env::temp_dir().join(format!("cardumen-deck-tailer-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("stream.log");
        let mut file = fs::File::create(&path).unwrap();
        writeln!(file, "line 1").unwrap();
        writeln!(file, "line 2").unwrap();
        file.flush().unwrap();

        let mut tailer = Tailer::open(&path, false).unwrap();
        assert_eq!(tailer.poll().unwrap(), vec!["line 1", "line 2"]);
        assert!(tailer.poll().unwrap().is_empty());

        writeln!(file, "line 3").unwrap();
        file.flush().unwrap();
        assert_eq!(tailer.poll().unwrap(), vec!["line 3"]);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn detects_fatal_error_in_tail() {
        let service_id = format!("test-{}", Uuid::new_v4());
        let dir = service_log_dir(&service_id).unwrap();
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("run.log");
        let mut file = fs::File::create(&path).unwrap();
        writeln!(file, "Starting development server").unwrap();
        writeln!(file, "ModuleNotFoundError: No module named 'corsheaders'").unwrap();

        let error = detect_fatal_error(&service_id, Some("run")).unwrap();
        assert!(error.contains("ModuleNotFoundError"));

        let _ = fs::remove_dir_all(dir);
    }
}
