use crate::modules::config::{RuntimeEntry, load_runtime, save_runtime};
use crate::modules::process::is_process_alive_with_created_at;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Config error: {0}")]
    Config(#[from] crate::modules::config::ConfigError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, RuntimeError>;

fn now_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn register_runtime(
    service_id: &str,
    pid: u32,
    run_id: &str,
    log_path: &str,
    process_created_at: Option<u64>,
) -> Result<()> {
    let mut runtime = load_runtime()?;
    runtime.retain(|entry| entry.service_id != service_id);
    runtime.push(RuntimeEntry {
        service_id: service_id.to_string(),
        pid,
        start_time: now_seconds(),
        run_id: run_id.to_string(),
        log_path: log_path.to_string(),
        status: "running".to_string(),
        exit_code: None,
        ended_at: None,
        last_error: None,
        process_created_at,
    });

    save_runtime(&runtime)?;
    Ok(())
}

pub fn mark_service_stopping(service_id: &str) -> Result<()> {
    let mut runtime = load_runtime()?;
    if let Some(entry) = runtime
        .iter_mut()
        .find(|entry| entry.service_id == service_id)
    {
        entry.status = "stopping".to_string();
        entry.last_error = None;
    }
    save_runtime(&runtime)?;
    Ok(())
}

pub fn mark_service_exited(service_id: &str, exit_code: Option<i32>) -> Result<()> {
    let mut runtime = load_runtime()?;
    if let Some(entry) = runtime
        .iter_mut()
        .find(|entry| entry.service_id == service_id)
    {
        entry.exit_code = exit_code;
        entry.ended_at = Some(now_seconds());

        if entry.status == "stopping" || entry.status == "stopped" || exit_code == Some(0) {
            entry.status = "stopped".to_string();
            entry.last_error = None;
        } else {
            entry.status = "error".to_string();
            entry.last_error = Some(match exit_code {
                Some(code) => format!("El proceso terminó con código {}", code),
                None => "El proceso terminó inesperadamente".to_string(),
            });
        }

        let service_name = crate::modules::config::load_config()
            .ok()
            .and_then(|config| {
                crate::modules::config::find_service(&config, service_id)
                    .map(|service| service.name.clone())
            })
            .unwrap_or_else(|| service_id.to_string());
        let context = serde_json::json!({
            "exitCode": exit_code,
            "runId": entry.run_id,
            "logPath": entry.log_path,
        });

        if entry.status == "error" {
            let message = entry
                .last_error
                .clone()
                .unwrap_or_else(|| "El proceso terminó inesperadamente".to_string());
            crate::modules::dashboard_log::record(
                crate::modules::dashboard_log::DashboardLogInput::error(
                    "process",
                    "service.exit",
                    format!("{}: {}", service_name, message),
                )
                .with_service(service_id, &service_name)
                .with_context(context),
            );
        } else {
            crate::modules::dashboard_log::record(
                crate::modules::dashboard_log::DashboardLogInput::info(
                    "process",
                    "service.exit",
                    format!("Proceso de {} finalizado", service_name),
                )
                .with_service(service_id, &service_name)
                .with_context(context),
            );
        }
    }
    save_runtime(&runtime)?;
    Ok(())
}

pub fn unregister_runtime(service_id: &str) -> Result<()> {
    let mut runtime = crate::modules::config::load_runtime()?;
    runtime.retain(|r| r.service_id != service_id);
    save_runtime(&runtime)?;
    Ok(())
}

pub fn mark_service_stopped(service_id: &str) -> Result<()> {
    let mut runtime = crate::modules::config::load_runtime()?;
    if let Some(entry) = runtime.iter_mut().find(|r| r.service_id == service_id) {
        entry.status = "stopped".to_string();
        entry.last_error = None;
        entry.exit_code = None;
    }
    save_runtime(&runtime)?;
    Ok(())
}

pub fn adopt_runtime_processes() -> Result<()> {
    let runtime = crate::modules::config::load_runtime()?;

    for entry in runtime {
        if entry.status == "running"
            && is_process_alive_with_created_at(entry.pid, entry.process_created_at)
        {
            // Check if already managed
            let processes = crate::modules::process::PROCESSES.lock().unwrap();
            if !processes.contains_key(&entry.service_id) {
                // This is an adopted process
                eprintln!(
                    "Adopting process {} for service {}",
                    entry.pid, entry.service_id
                );
            }
        }
    }

    Ok(())
}

pub fn runtime_reconcile_runtime() -> Result<()> {
    let mut runtime = crate::modules::config::load_runtime()?;

    for entry in &mut runtime {
        if entry.status == "running" || entry.status == "stopping" {
            if is_process_alive_with_created_at(entry.pid, entry.process_created_at) {
                continue;
            }

            if let Some(error) =
                crate::modules::logs::detect_fatal_error(&entry.service_id, Some(&entry.run_id))
            {
                entry.status = "error".to_string();
                entry.last_error = Some(error);
            } else {
                entry.status = "stopped".to_string();
                entry.last_error = None;
            }
            entry.ended_at = Some(now_seconds());
        }
    }

    // Remove entries older than 30 days that are stopped
    let cutoff = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - (30 * 86400);

    runtime.retain(|r| !(r.status == "stopped" && r.start_time < cutoff));

    crate::modules::config::save_runtime(&runtime)?;
    Ok(())
}
