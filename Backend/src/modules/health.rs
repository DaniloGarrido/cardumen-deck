use crate::modules::config::{RuntimeEntry, Service, Settings, load_config, load_runtime};
use crate::modules::dashboard_log::{DashboardLogInput, record};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net::TcpStream;
use tokio::time::timeout;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthStatus {
    pub state: String,
    pub kind: String,
    pub target: Option<String>,
    pub checked_at: Option<u64>,
    pub latency_ms: Option<u64>,
    pub consecutive_failures: u32,
    pub message: Option<String>,
}

impl HealthStatus {
    fn new(state: &str) -> Self {
        Self {
            state: state.to_string(),
            kind: "process".to_string(),
            target: None,
            checked_at: None,
            latency_ms: None,
            consecutive_failures: 0,
            message: None,
        }
    }
}

#[derive(Debug, Clone)]
struct HealthEntry {
    status: HealthStatus,
    last_started: Option<Instant>,
    in_flight: bool,
}

impl Default for HealthEntry {
    fn default() -> Self {
        Self {
            status: HealthStatus::new("checking"),
            last_started: None,
            in_flight: false,
        }
    }
}

static HEALTH_STATE: Lazy<Mutex<HashMap<String, HealthEntry>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CheckTarget {
    Http(String),
    Tcp(String),
    Process,
}

impl CheckTarget {
    fn kind(&self) -> &'static str {
        match self {
            Self::Http(_) => "http",
            Self::Tcp(_) => "tcp",
            Self::Process => "process",
        }
    }

    fn target(&self) -> Option<String> {
        match self {
            Self::Http(url) => Some(url.clone()),
            Self::Tcp(target) => Some(target.clone()),
            Self::Process => None,
        }
    }
}

fn resolve_target(service: &Service) -> CheckTarget {
    if let Some(url) = service.url.as_deref() {
        if let Ok(parsed) = reqwest::Url::parse(url) {
            if matches!(parsed.scheme(), "http" | "https") {
                return CheckTarget::Http(url.to_string());
            }
        }
    }

    if let Some(port) = service.port {
        return CheckTarget::Tcp(format!("127.0.0.1:{port}"));
    }

    if let Some(url) = service.url.as_deref() {
        if let Ok(parsed) = reqwest::Url::parse(url) {
            if let Some(port) = parsed.port_or_known_default() {
                let host = parsed.host_str().unwrap_or("127.0.0.1");
                let target = if host.contains(':') {
                    format!("[{host}]:{port}")
                } else {
                    format!("{host}:{port}")
                };
                return CheckTarget::Tcp(target);
            }
        }
    }

    CheckTarget::Process
}

async fn check_target(
    client: &reqwest::Client,
    target: &CheckTarget,
    timeout_secs: u64,
) -> Result<u64, String> {
    let started = Instant::now();
    let request_timeout = Duration::from_secs(timeout_secs.max(1));

    match target {
        CheckTarget::Http(url) => {
            let response = client
                .get(url)
                .timeout(request_timeout)
                .send()
                .await
                .map_err(|error| error.to_string())?;
            if response.status().as_u16() >= 500 {
                return Err(format!("HTTP {}", response.status()));
            }
        }
        CheckTarget::Tcp(target) => {
            timeout(request_timeout, TcpStream::connect(target))
                .await
                .map_err(|_| "Tiempo de espera agotado".to_string())?
                .map_err(|error| error.to_string())?;
        }
        CheckTarget::Process => {}
    }

    Ok(started.elapsed().as_millis() as u64)
}

fn set_state(service_id: &str, state: &str) {
    let mut states = HEALTH_STATE.lock().unwrap();
    let entry = states.entry(service_id.to_string()).or_default();

    if entry.status.state != state {
        entry.status = HealthStatus::new(state);
    }
    entry.in_flight = false;
    entry.last_started = None;
}

fn apply_result(
    service_id: &str,
    service_name: &str,
    target: CheckTarget,
    result: Result<u64, String>,
    failure_threshold: u32,
) {
    let checked_at = now_secs();
    let (previous, next, failures, latency, message) = {
        let mut states = HEALTH_STATE.lock().unwrap();
        let entry = states.entry(service_id.to_string()).or_default();
        entry.in_flight = false;

        let previous = entry.status.state.clone();
        entry.status.checked_at = Some(checked_at);
        entry.status.kind = target.kind().to_string();
        entry.status.target = target.target();

        match result {
            Ok(latency) => {
                entry.status.latency_ms = Some(latency);
                entry.status.consecutive_failures = 0;
                entry.status.state = "healthy".to_string();
                entry.status.message = None;
            }
            Err(message) => {
                entry.status.latency_ms = None;
                entry.status.consecutive_failures =
                    entry.status.consecutive_failures.saturating_add(1);
                entry.status.message = Some(message);
                entry.status.state =
                    if entry.status.consecutive_failures >= failure_threshold.max(1) {
                        "unhealthy".to_string()
                    } else {
                        "degraded".to_string()
                    };
            }
        }

        (
            previous,
            entry.status.state.clone(),
            entry.status.consecutive_failures,
            entry.status.latency_ms,
            entry.status.message.clone(),
        )
    };

    if previous == next {
        return;
    }

    let context = serde_json::json!({
        "kind": target.kind(),
        "target": target.target(),
        "latencyMs": latency,
        "consecutiveFailures": failures,
    });

    if next == "unhealthy" {
        record(
            DashboardLogInput::warn(
                "health",
                "service.unhealthy",
                format!("{} no responde", service_name),
            )
            .with_service(service_id, service_name)
            .with_details(message.unwrap_or_else(|| "Sin respuesta del health check".to_string()))
            .with_context(context),
        );
    } else if next == "healthy" && (previous == "unhealthy" || previous == "degraded") {
        record(
            DashboardLogInput::info(
                "health",
                "service.recovered",
                format!("{} volvió a responder", service_name),
            )
            .with_service(service_id, service_name)
            .with_context(context),
        );
    }
}

fn start_check(client: reqwest::Client, service: Service, settings: Settings, interval: Duration) {
    let target = resolve_target(&service);
    let service_id = service.id.clone();
    let service_name = service.name.clone();
    let timeout_secs = settings.health_check_timeout_secs;
    let failure_threshold = settings.health_check_failures;

    let should_start = {
        let mut states = HEALTH_STATE.lock().unwrap();
        let entry = states.entry(service_id.clone()).or_default();
        if entry.in_flight {
            false
        } else if entry
            .last_started
            .map(|last| last.elapsed() >= interval)
            .unwrap_or(true)
        {
            entry.in_flight = true;
            entry.last_started = Some(Instant::now());
            if entry.status.state == "stopped" || entry.status.state == "unknown" {
                entry.status = HealthStatus::new("checking");
            }
            true
        } else {
            false
        }
    };

    if !should_start {
        return;
    }

    tauri::async_runtime::spawn(async move {
        let result = check_target(&client, &target, timeout_secs).await;
        apply_result(
            &service_id,
            &service_name,
            target,
            result,
            failure_threshold,
        );
    });
}

fn monitor_service(
    client: &reqwest::Client,
    service: &Service,
    settings: &Settings,
    runtime: &[RuntimeEntry],
) {
    if !crate::modules::process::is_service_alive(&service.id) {
        set_state(&service.id, "stopped");
        return;
    }

    let entry = runtime.iter().find(|entry| entry.service_id == service.id);
    if entry
        .map(|entry| entry.status == "stopping" || entry.status == "stopped")
        .unwrap_or(false)
    {
        set_state(&service.id, "stopped");
        return;
    }

    if let Some(entry) = entry {
        let started_at = entry.start_time;
        if now_secs().saturating_sub(started_at) < settings.health_check_startup_grace_secs {
            set_state(&service.id, "checking");
            return;
        }
    }

    let interval = Duration::from_secs(settings.health_check_interval_secs.max(1));
    start_check(client.clone(), service.clone(), settings.clone(), interval);
}

pub fn start_monitor() {
    tauri::async_runtime::spawn(async move {
        let client = reqwest::Client::builder()
            .user_agent("CardumenDeck/0.1")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        loop {
            match load_config() {
                Ok(config) => {
                    if !config.settings.health_check_enabled {
                        HEALTH_STATE.lock().unwrap().clear();
                    } else {
                        let runtime = load_runtime().unwrap_or_default();
                        for service in &config.services {
                            monitor_service(&client, service, &config.settings, &runtime);
                        }
                    }
                }
                Err(error) => {
                    eprintln!("No se pudo cargar la configuración para health checks: {error}");
                }
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
}

pub fn snapshot(service_id: &str) -> Option<HealthStatus> {
    HEALTH_STATE
        .lock()
        .unwrap()
        .get(service_id)
        .map(|entry| entry.status.clone())
}

pub fn reset(service_id: &str, state: &str) {
    set_state(service_id, state);
}

pub fn remove(service_id: &str) {
    HEALTH_STATE.lock().unwrap().remove(service_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::config::Service;
    use chrono::Utc;
    use std::collections::HashMap;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn service(url: Option<&str>, port: Option<u16>) -> Service {
        Service {
            id: "service".to_string(),
            workspace_id: "default".to_string(),
            name: "Service".to_string(),
            category: "custom".to_string(),
            preset: "custom".to_string(),
            cwd: String::new(),
            command: String::new(),
            args: Vec::new(),
            setup_command: None,
            env: HashMap::new(),
            url: url.map(str::to_string),
            port,
            color: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn resolves_http_target_from_url() {
        assert_eq!(
            resolve_target(&service(Some("http://localhost:5173"), None)),
            CheckTarget::Http("http://localhost:5173".to_string())
        );
    }

    #[test]
    fn resolves_tcp_target_from_port() {
        assert_eq!(
            resolve_target(&service(None, Some(8080))),
            CheckTarget::Tcp("127.0.0.1:8080".to_string())
        );
    }

    #[test]
    fn falls_back_to_process_target() {
        assert_eq!(resolve_target(&service(None, None)), CheckTarget::Process);
    }

    #[tokio::test]
    async fn http_check_accepts_successful_response() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buffer = [0u8; 1024];
                let _ = stream.read(&mut buffer).await;
                let _ = stream
                    .write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK",
                    )
                    .await;
            }
        });

        let client = reqwest::Client::new();
        let result =
            check_target(&client, &CheckTarget::Http(format!("http://{address}")), 2).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn http_check_rejects_server_error() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buffer = [0u8; 1024];
                let _ = stream.read(&mut buffer).await;
                let _ = stream
                    .write_all(
                        b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    )
                    .await;
            }
        });

        let client = reqwest::Client::new();
        let result =
            check_target(&client, &CheckTarget::Http(format!("http://{address}")), 2).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn tcp_check_connects_to_open_port() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        let client = reqwest::Client::new();
        let result = check_target(&client, &CheckTarget::Tcp(address.to_string()), 2).await;

        assert!(result.is_ok());
    }
}
