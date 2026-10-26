use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use thiserror::Error;
use uuid::Uuid;

static WRITE_LOCK: once_cell::sync::Lazy<Mutex<()>> = once_cell::sync::Lazy::new(|| Mutex::new(()));

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Invalid config: {0}")]
    Invalid(String),
}

pub const DEFAULT_WORKSPACE_ID: &str = "default";
const APP_DATA_DIR: &str = "CardumenDeck";
const LEGACY_APP_DATA_DIR: &str = "ServiceDashboard";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde_with::serde_as]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    #[serde_as(as = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde_as(as = "chrono::serde::ts_seconds")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde_with::serde_as]
pub struct Service {
    pub id: String,
    #[serde(default)]
    pub workspace_id: String,
    pub name: String,
    pub category: String,
    pub preset: String,
    pub cwd: String,
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub setup_command: Option<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub url: Option<String>,
    pub port: Option<u16>,
    pub color: Option<String>,
    #[serde_as(as = "chrono::serde::ts_seconds")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde_as(as = "chrono::serde::ts_seconds")]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub confirm_on_exit: bool,
    pub log_retention_runs: usize,
    pub log_retention_days: usize,
    pub health_check_enabled: bool,
    pub health_check_interval_secs: u64,
    pub health_check_timeout_secs: u64,
    pub health_check_failures: u32,
    pub health_check_startup_grace_secs: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            confirm_on_exit: true,
            log_retention_runs: 10,
            log_retention_days: 30,
            health_check_enabled: true,
            health_check_interval_secs: 10,
            health_check_timeout_secs: 3,
            health_check_failures: 3,
            health_check_startup_grace_secs: 15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub schema_version: u32,
    #[serde(default)]
    pub workspaces: Vec<Workspace>,
    #[serde(default)]
    pub active_workspace_id: Option<String>,
    pub services: Vec<Service>,
    pub settings: Settings,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema_version: 2,
            workspaces: Vec::new(),
            active_workspace_id: None,
            services: Vec::new(),
            settings: Settings::default(),
        }
    }
}

pub fn default_workspace() -> Workspace {
    let now = chrono::Utc::now();
    Workspace {
        id: DEFAULT_WORKSPACE_ID.to_string(),
        name: "General".to_string(),
        color: Some("#4f46e5".to_string()),
        created_at: now,
        updated_at: now,
    }
}

pub fn normalize_config(mut config: Config) -> (Config, bool) {
    let mut changed = false;

    if config.workspaces.is_empty() {
        config.workspaces.push(default_workspace());
        changed = true;
    }

    if config.schema_version != 2 {
        config.schema_version = 2;
        changed = true;
    }

    let workspace_ids: Vec<String> = config.workspaces.iter().map(|w| w.id.clone()).collect();
    let active_is_valid = config
        .active_workspace_id
        .as_ref()
        .map(|id| workspace_ids.iter().any(|workspace_id| workspace_id == id))
        .unwrap_or(false);

    if !active_is_valid {
        config.active_workspace_id = config.workspaces.first().map(|w| w.id.clone());
        changed = true;
    }

    let fallback_workspace_id = config
        .active_workspace_id
        .clone()
        .or_else(|| config.workspaces.first().map(|w| w.id.clone()))
        .unwrap_or_else(|| DEFAULT_WORKSPACE_ID.to_string());

    for service in &mut config.services {
        let valid = config
            .workspaces
            .iter()
            .any(|workspace| workspace.id == service.workspace_id);
        if !valid {
            service.workspace_id = fallback_workspace_id.clone();
            changed = true;
        }
    }

    (config, changed)
}

fn config_dir() -> Result<PathBuf, ConfigError> {
    let base = dirs_next::data_dir()
        .ok_or_else(|| ConfigError::Invalid("Could not find data directory".into()))?;
    Ok(base.join(APP_DATA_DIR))
}

fn legacy_config_dir() -> Result<PathBuf, ConfigError> {
    let base = dirs_next::data_dir()
        .ok_or_else(|| ConfigError::Invalid("Could not find data directory".into()))?;
    Ok(base.join(LEGACY_APP_DATA_DIR))
}

fn config_path() -> Result<PathBuf, ConfigError> {
    Ok(config_dir()?.join("config.json"))
}

fn runtime_path() -> Result<PathBuf, ConfigError> {
    Ok(config_dir()?.join("runtime.json"))
}

fn write_atomic(path: &Path, content: &str) -> Result<(), ConfigError> {
    let _guard = WRITE_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "data".to_string());
    let temp_path = path.with_file_name(format!(".{file_name}.{}.tmp", Uuid::new_v4()));
    fs::write(&temp_path, content)?;

    if let Err(error) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(ConfigError::Io(error));
    }

    Ok(())
}

fn parse_json_prefix<T: DeserializeOwned>(content: &str) -> Option<T> {
    let mut deserializer = serde_json::Deserializer::from_str(content);
    T::deserialize(&mut deserializer).ok()
}

fn logs_dir() -> Result<PathBuf, ConfigError> {
    let base = dirs_next::data_local_dir()
        .ok_or_else(|| ConfigError::Invalid("Could not find local data directory".into()))?;
    Ok(base.join(APP_DATA_DIR).join("logs"))
}

fn legacy_logs_dir() -> Result<PathBuf, ConfigError> {
    let base = dirs_next::data_local_dir()
        .ok_or_else(|| ConfigError::Invalid("Could not find local data directory".into()))?;
    Ok(base.join(LEGACY_APP_DATA_DIR).join("logs"))
}

fn copy_if_exists(source: &Path, destination: &Path) -> Result<bool, ConfigError> {
    if !source.exists() {
        return Ok(false);
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, destination)?;
    Ok(true)
}

fn migrate_legacy_data_from(
    config_dir: &Path,
    legacy_config_dir: &Path,
    logs_dir: &Path,
    legacy_logs_dir: &Path,
) -> Result<bool, ConfigError> {
    let config_path = config_dir.join("config.json");
    let legacy_config_path = legacy_config_dir.join("config.json");
    if config_path.exists() || !legacy_config_path.exists() {
        return Ok(false);
    }

    fs::create_dir_all(config_dir)?;
    copy_if_exists(&legacy_config_path, &config_path)?;
    copy_if_exists(
        &legacy_config_dir.join("runtime.json"),
        &config_dir.join("runtime.json"),
    )?;

    let legacy_dashboard = legacy_logs_dir.join("_dashboard").join("dashboard.jsonl");
    let dashboard = logs_dir.join("_dashboard").join("dashboard.jsonl");
    copy_if_exists(&legacy_dashboard, &dashboard)?;

    Ok(true)
}

pub fn migrate_legacy_data() -> Result<bool, ConfigError> {
    migrate_legacy_data_from(
        &config_dir()?,
        &legacy_config_dir()?,
        &logs_dir()?,
        &legacy_logs_dir()?,
    )
}

pub fn ensure_dirs() -> Result<(), ConfigError> {
    fs::create_dir_all(config_dir()?)?;
    fs::create_dir_all(logs_dir()?)?;
    Ok(())
}

pub fn load_config() -> Result<Config, ConfigError> {
    ensure_dirs()?;
    let path = config_path()?;
    let (config, repaired) = if !path.exists() {
        (Config::default(), false)
    } else {
        let content = fs::read_to_string(&path)?;
        match serde_json::from_str::<Config>(&content) {
            Ok(config) => (config, false),
            Err(original_error) => {
                let Some(config) = parse_json_prefix::<Config>(&content) else {
                    return Err(ConfigError::Serde(original_error));
                };
                crate::modules::dashboard_log::record(
                    crate::modules::dashboard_log::DashboardLogInput::warn(
                        "backend",
                        "config.repair",
                        "Se recuperó config.json con datos sobrantes",
                    )
                    .with_details(original_error.to_string()),
                );
                (config, true)
            }
        }
    };

    let (config, changed) = normalize_config(config);
    if changed || repaired {
        save_config(&config)?;
    }
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), ConfigError> {
    ensure_dirs()?;
    let path = config_path()?;
    let content = serde_json::to_string_pretty(config)?;
    write_atomic(&path, &content)
}

pub fn find_workspace<'a>(config: &'a Config, id: &'a str) -> Option<&'a Workspace> {
    config
        .workspaces
        .iter()
        .find(|workspace| workspace.id == id)
}

pub fn find_service<'a>(config: &'a Config, id: &'a str) -> Option<&'a Service> {
    config.services.iter().find(|s| s.id == id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEntry {
    pub service_id: String,
    pub pid: u32,
    pub start_time: u64,
    pub run_id: String,
    pub log_path: String,
    pub status: String, // "running", "stopping", "stopped", "error"
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub ended_at: Option<u64>,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub process_created_at: Option<u64>,
}

pub fn load_runtime() -> Result<Vec<RuntimeEntry>, ConfigError> {
    let path = runtime_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path)?;
    match serde_json::from_str::<Vec<RuntimeEntry>>(&content) {
        Ok(entries) => Ok(entries),
        Err(original_error) => {
            let Some(entries) = parse_json_prefix::<Vec<RuntimeEntry>>(&content) else {
                return Err(ConfigError::Serde(original_error));
            };
            let _ = save_runtime(&entries);
            crate::modules::dashboard_log::record(
                crate::modules::dashboard_log::DashboardLogInput::warn(
                    "backend",
                    "runtime.repair",
                    "Se recuperó runtime.json con datos sobrantes",
                )
                .with_details(original_error.to_string()),
            );
            Ok(entries)
        }
    }
}

pub fn save_runtime(entries: &[RuntimeEntry]) -> Result<(), ConfigError> {
    ensure_dirs()?;
    let path = runtime_path()?;
    let content = serde_json::to_string_pretty(entries)?;
    write_atomic(&path, &content)
}

pub fn get_logs_dir() -> Result<PathBuf, ConfigError> {
    logs_dir()
}

pub fn service_log_dir(service_id: &str) -> Result<PathBuf, ConfigError> {
    Ok(logs_dir()?.join(service_id))
}

pub fn setup_logs_dir() -> Result<PathBuf, ConfigError> {
    Ok(logs_dir()?.join("_setup"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInput {
    #[serde(default)]
    pub workspace_id: Option<String>,
    pub name: String,
    pub category: String,
    pub preset: String,
    pub cwd: String,
    pub command: String,
    pub args: Vec<String>,
    pub setup_command: Option<String>,
    pub env: HashMap<String, String>,
    pub url: Option<String>,
    pub port: Option<u16>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogOutput {
    pub run_id: String,
    pub lines: Vec<String>,
    pub truncated: bool,
    #[serde(default)]
    pub total_lines: usize,
    #[serde(default)]
    pub updated_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunInfo {
    pub run_id: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub status: String,
    pub log_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    pub cwd_exists: bool,
    pub command_exists: bool,
    pub port_available: bool,
    pub wrapper_found: bool,
    pub warnings: Vec<String>,
}
#[cfg(test)]
mod tests {
    use super::*;

    fn service(id: &str) -> Service {
        let now = chrono::Utc::now();
        Service {
            id: id.to_string(),
            workspace_id: String::new(),
            name: id.to_string(),
            category: "custom".to_string(),
            preset: "custom".to_string(),
            cwd: String::new(),
            command: String::new(),
            args: Vec::new(),
            setup_command: None,
            env: HashMap::new(),
            url: None,
            port: None,
            color: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn migrates_old_services_to_default_workspace() {
        let config = Config {
            schema_version: 1,
            workspaces: Vec::new(),
            active_workspace_id: None,
            services: vec![service("service-1")],
            settings: Settings::default(),
        };

        let (config, changed) = normalize_config(config);
        assert!(changed);
        assert_eq!(config.schema_version, 2);
        assert_eq!(config.workspaces.len(), 1);
        assert_eq!(config.workspaces[0].id, DEFAULT_WORKSPACE_ID);
        assert_eq!(
            config.active_workspace_id.as_deref(),
            Some(DEFAULT_WORKSPACE_ID)
        );
        assert_eq!(config.services[0].workspace_id, DEFAULT_WORKSPACE_ID);
    }

    #[test]
    fn parses_valid_prefix_with_trailing_garbage() {
        let config = Config::default();
        let content = format!("{}]\n", serde_json::to_string(&config).unwrap());
        let parsed = parse_json_prefix::<Config>(&content).expect("recover valid prefix");
        assert_eq!(parsed.schema_version, 2);
    }

    #[test]
    fn fixes_unknown_active_workspace() {
        let mut config = Config {
            schema_version: 2,
            workspaces: vec![default_workspace()],
            active_workspace_id: Some("missing".to_string()),
            services: vec![service("service-1")],
            settings: Settings::default(),
        };
        config.services[0].workspace_id = DEFAULT_WORKSPACE_ID.to_string();

        let (config, changed) = normalize_config(config);
        assert!(changed);
        assert_eq!(
            config.active_workspace_id.as_deref(),
            Some(DEFAULT_WORKSPACE_ID)
        );
    }

    #[test]
    fn settings_missing_health_fields_use_defaults() {
        let settings: Settings = serde_json::from_str(
            r#"{"confirmOnExit":true,"logRetentionRuns":10,"logRetentionDays":30}"#,
        )
        .expect("old settings should deserialize");

        assert!(settings.health_check_enabled);
        assert_eq!(settings.health_check_interval_secs, 10);
        assert_eq!(settings.health_check_timeout_secs, 3);
        assert_eq!(settings.health_check_failures, 3);
        assert_eq!(settings.health_check_startup_grace_secs, 15);
    }

    #[test]
    fn migrates_legacy_config_and_dashboard_log() {
        let root = std::env::temp_dir().join(format!("cardumen-deck-migration-{}", Uuid::new_v4()));
        let new_config_dir = root.join("new");
        let legacy_config_dir = root.join("legacy");
        let new_logs_dir = root.join("new-logs");
        let legacy_logs_dir = root.join("legacy-logs");

        fs::create_dir_all(&legacy_config_dir).expect("create legacy config dir");
        fs::create_dir_all(legacy_logs_dir.join("_dashboard")).expect("create legacy logs dir");
        fs::write(legacy_config_dir.join("config.json"), "{}").expect("write config");
        fs::write(legacy_config_dir.join("runtime.json"), "[]").expect("write runtime");
        fs::write(
            legacy_logs_dir.join("_dashboard").join("dashboard.jsonl"),
            "entry\n",
        )
        .expect("write dashboard log");

        assert!(
            migrate_legacy_data_from(
                &new_config_dir,
                &legacy_config_dir,
                &new_logs_dir,
                &legacy_logs_dir,
            )
            .expect("migrate legacy data")
        );
        assert!(new_config_dir.join("config.json").exists());
        assert!(new_config_dir.join("runtime.json").exists());
        assert!(
            new_logs_dir
                .join("_dashboard")
                .join("dashboard.jsonl")
                .exists()
        );
        assert!(
            !migrate_legacy_data_from(
                &new_config_dir,
                &legacy_config_dir,
                &new_logs_dir,
                &legacy_logs_dir,
            )
            .expect("second migration should be a no-op")
        );

        let _ = fs::remove_dir_all(root);
    }
}
