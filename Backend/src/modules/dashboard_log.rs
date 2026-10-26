use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

const LOG_FILE_NAME: &str = "dashboard.jsonl";
const BACKUP_FILE_NAME: &str = "dashboard.jsonl.1";
const MAX_LOG_BYTES: u64 = 2 * 1024 * 1024;
const DEFAULT_READ_LIMIT: usize = 500;
const MAX_READ_LIMIT: usize = 2000;

static LOG_LOCK: once_cell::sync::Lazy<Mutex<()>> = once_cell::sync::Lazy::new(|| Mutex::new(()));

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardLogInput {
    #[serde(default)]
    pub level: String,
    #[serde(default)]
    pub source: String,
    pub scope: String,
    pub message: String,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub service_id: Option<String>,
    #[serde(default)]
    pub service_name: Option<String>,
    #[serde(default)]
    pub context: Option<Value>,
}

impl DashboardLogInput {
    pub fn error(source: &str, scope: &str, message: impl Into<String>) -> Self {
        Self {
            level: "error".into(),
            source: source.into(),
            scope: scope.into(),
            message: message.into(),
            ..Self::default()
        }
    }

    pub fn warn(source: &str, scope: &str, message: impl Into<String>) -> Self {
        Self {
            level: "warn".into(),
            source: source.into(),
            scope: scope.into(),
            message: message.into(),
            ..Self::default()
        }
    }

    pub fn info(source: &str, scope: &str, message: impl Into<String>) -> Self {
        Self {
            level: "info".into(),
            source: source.into(),
            scope: scope.into(),
            message: message.into(),
            ..Self::default()
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        let details = details.into();
        if !details.trim().is_empty() {
            self.details = Some(details);
        }
        self
    }

    pub fn with_service(mut self, id: impl Into<String>, name: impl Into<String>) -> Self {
        self.service_id = Some(id.into());
        self.service_name = Some(name.into());
        self
    }

    pub fn with_context(mut self, context: Value) -> Self {
        self.context = Some(context);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardLogEntry {
    pub id: String,
    pub timestamp: String,
    pub level: String,
    pub source: String,
    pub scope: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardLogPage {
    pub entries: Vec<DashboardLogEntry>,
    pub total: usize,
    pub path: String,
    pub size_bytes: u64,
    pub truncated: bool,
}

fn log_dir() -> Result<PathBuf, String> {
    Ok(crate::modules::config::get_logs_dir()
        .map_err(|error| error.to_string())?
        .join("_dashboard"))
}

fn log_path() -> Result<PathBuf, String> {
    Ok(log_dir()?.join(LOG_FILE_NAME))
}

fn ensure_log_dir() -> Result<PathBuf, String> {
    let dir = log_dir()?;
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

fn normalize_level(level: &str) -> String {
    match level.trim().to_ascii_lowercase().as_str() {
        "warn" | "warning" => "warn".into(),
        "info" | "information" => "info".into(),
        _ => "error".into(),
    }
}

fn append_entry(path: &Path, input: DashboardLogInput) -> Result<DashboardLogEntry, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    rotate_if_needed(path, MAX_LOG_BYTES);

    let entry = DashboardLogEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        level: normalize_level(&input.level),
        source: if input.source.trim().is_empty() {
            "backend".into()
        } else {
            input.source
        },
        scope: input.scope,
        message: input.message,
        details: input.details,
        service_id: input.service_id,
        service_name: input.service_name,
        context: input.context,
    };

    let line = serde_json::to_string(&entry).map_err(|error| error.to_string())?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    writeln!(file, "{line}").map_err(|error| error.to_string())?;
    file.flush().map_err(|error| error.to_string())?;

    Ok(entry)
}

fn rotate_if_needed(path: &Path, max_bytes: u64) {
    let size = fs::metadata(path)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    if size < max_bytes {
        return;
    }

    let backup = path.with_file_name(BACKUP_FILE_NAME);
    if backup.exists() {
        let _ = fs::remove_file(&backup);
    }
    let _ = fs::rename(path, backup);
}

fn read_entries(
    path: &Path,
    limit: usize,
    level: Option<&str>,
    query: Option<&str>,
) -> Result<DashboardLogPage, String> {
    let limit = limit.clamp(1, MAX_READ_LIMIT);
    let level_filter = level
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "all")
        .map(normalize_level);
    let query_filter = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_lowercase());

    if !path.exists() {
        return Ok(DashboardLogPage {
            entries: Vec::new(),
            total: 0,
            path: path.to_string_lossy().to_string(),
            size_bytes: 0,
            truncated: false,
        });
    }

    let content = fs::read_to_string(path).unwrap_or_default();
    let size_bytes = fs::metadata(path)
        .map(|metadata| metadata.len())
        .unwrap_or(0);
    let mut entries = Vec::new();

    for line in content.lines().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let Ok(entry) = serde_json::from_str::<DashboardLogEntry>(line) else {
            continue;
        };

        if let Some(level) = level_filter.as_ref() {
            if entry.level != *level {
                continue;
            }
        }

        if let Some(query) = query_filter.as_ref() {
            let searchable = [
                entry.message.as_str(),
                entry.details.as_deref().unwrap_or_default(),
                entry.scope.as_str(),
                entry.source.as_str(),
                entry.service_name.as_deref().unwrap_or_default(),
            ]
            .join(" ")
            .to_lowercase();
            if !searchable.contains(query) {
                continue;
            }
        }

        entries.push(entry);
    }

    let total = entries.len();
    let truncated = total > limit;
    entries.truncate(limit);

    Ok(DashboardLogPage {
        entries,
        total,
        path: path.to_string_lossy().to_string(),
        size_bytes,
        truncated,
    })
}

pub fn write(input: DashboardLogInput) -> Result<DashboardLogEntry, String> {
    ensure_log_dir()?;
    let path = log_path()?;
    let _guard = LOG_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    append_entry(&path, input)
}

pub fn record(input: DashboardLogInput) {
    if let Err(error) = write(input) {
        eprintln!("No se pudo escribir el registro del dashboard: {error}");
    }
}

pub fn read(
    limit: Option<usize>,
    level: Option<String>,
    query: Option<String>,
) -> Result<DashboardLogPage, String> {
    ensure_log_dir()?;
    let path = log_path()?;
    let _guard = LOG_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    read_entries(
        &path,
        limit.unwrap_or(DEFAULT_READ_LIMIT),
        level.as_deref(),
        query.as_deref(),
    )
}

pub fn clear() -> Result<(), String> {
    let dir = ensure_log_dir()?;
    let _guard = LOG_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for path in [dir.join(LOG_FILE_NAME), dir.join(BACKUP_FILE_NAME)] {
        if path.exists() {
            fs::remove_file(path).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

pub fn folder() -> Result<PathBuf, String> {
    ensure_log_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file() -> PathBuf {
        std::env::temp_dir().join(format!("cardumen-deck-log-{}", Uuid::new_v4()))
    }

    #[test]
    fn writes_and_reads_filtered_entries() {
        let path = temp_file();
        append_entry(
            &path,
            DashboardLogInput::error("backend", "service.start", "No se pudo iniciar")
                .with_service("svc-1", "K-riko")
                .with_details("detalle"),
        )
        .expect("write error entry");
        append_entry(
            &path,
            DashboardLogInput::info("system", "app.start", "Aplicación iniciada"),
        )
        .expect("write info entry");

        let page = read_entries(&path, 10, Some("error"), None).expect("read entries");
        assert_eq!(page.total, 1);
        assert_eq!(page.entries[0].message, "No se pudo iniciar");
        assert_eq!(page.entries[0].service_name.as_deref(), Some("K-riko"));

        let page = read_entries(&path, 10, None, Some("K-riko")).expect("search entries");
        assert_eq!(page.total, 1);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn rotates_backup_when_limit_is_reached() {
        let path = temp_file();
        fs::write(&path, "existing\n").expect("seed log");
        rotate_if_needed(&path, 1);
        let backup = path.with_file_name(BACKUP_FILE_NAME);
        assert!(!path.exists());
        assert!(backup.exists());
        let _ = fs::remove_file(backup);
    }
}
