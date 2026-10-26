use crate::modules::config::{Diagnostics, Settings, find_service, load_config, save_config};
use crate::modules::dashboard_log::DashboardLogInput;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use uuid::Uuid;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hola, {}!", name)
}

#[tauri::command]
pub async fn get_services() -> Result<Vec<crate::modules::config::Service>, String> {
    let config = load_config().map_err(|e| e.to_string())?;
    Ok(config.services)
}

#[tauri::command]
pub async fn get_git_branches(paths: Vec<String>) -> Result<HashMap<String, String>, String> {
    Ok(crate::modules::git::branches_for(&paths))
}

async fn git_blocking<T, F>(operation: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, crate::modules::git::GitError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_git_repositories(
    paths: Vec<String>,
) -> Result<Vec<crate::modules::git::GitRepositorySummary>, String> {
    git_blocking(move || Ok(crate::modules::git::repositories_for(&paths))).await
}

#[tauri::command]
pub async fn get_git_state(path: String) -> Result<crate::modules::git::GitState, String> {
    git_blocking(move || crate::modules::git::state_for(&path)).await
}

#[tauri::command]
pub async fn git_fetch(path: String) -> Result<crate::modules::git::GitOperationResult, String> {
    git_blocking(move || crate::modules::git::fetch(&path)).await
}

#[tauri::command]
pub async fn git_pull(
    path: String,
    mode: String,
) -> Result<crate::modules::git::GitOperationResult, String> {
    git_blocking(move || crate::modules::git::pull(&path, &mode)).await
}

#[tauri::command]
pub async fn git_checkout(
    path: String,
    branch: String,
) -> Result<crate::modules::git::GitOperationResult, String> {
    git_blocking(move || crate::modules::git::checkout(&path, &branch)).await
}

#[tauri::command]
pub async fn git_create_branch(
    path: String,
    name: String,
    start_point: Option<String>,
) -> Result<crate::modules::git::GitOperationResult, String> {
    git_blocking(move || crate::modules::git::create_branch(&path, &name, start_point.as_deref()))
        .await
}

#[tauri::command]
pub async fn git_delete_branch(
    path: String,
    name: String,
) -> Result<crate::modules::git::GitOperationResult, String> {
    git_blocking(move || crate::modules::git::delete_branch(&path, &name)).await
}

#[tauri::command]
pub async fn git_merge(
    path: String,
    source: String,
    target: String,
) -> Result<crate::modules::git::GitOperationResult, String> {
    git_blocking(move || crate::modules::git::merge(&path, &source, &target)).await
}

#[tauri::command]
pub async fn git_abort_operation(
    path: String,
) -> Result<crate::modules::git::GitOperationResult, String> {
    git_blocking(move || crate::modules::git::abort_operation(&path)).await
}

#[tauri::command]
pub async fn git_push(
    path: String,
    remote: Option<String>,
) -> Result<crate::modules::git::GitOperationResult, String> {
    git_blocking(move || crate::modules::git::push(&path, remote)).await
}

#[tauri::command]
pub async fn get_service_dependencies(
    cwd: String,
) -> Result<crate::modules::dependencies::ServiceDependencyReport, String> {
    tauri::async_runtime::spawn_blocking(move || crate::modules::dependencies::detect(&cwd))
        .await
        .map_err(|error| error.to_string())?
}

fn resolve_workspace_id(
    config: &crate::modules::config::Config,
    requested: Option<String>,
) -> Result<String, String> {
    let workspace_id = requested
        .filter(|id| !id.trim().is_empty())
        .or_else(|| config.active_workspace_id.clone())
        .unwrap_or_default();

    if config
        .workspaces
        .iter()
        .any(|workspace| workspace.id == workspace_id)
    {
        Ok(workspace_id)
    } else {
        Err("Workspace no encontrado".to_string())
    }
}

#[tauri::command]
pub async fn get_workspace_state() -> Result<crate::modules::workspaces::WorkspaceState, String> {
    crate::modules::workspaces::get_workspace_state()
}

#[tauri::command]
pub async fn create_workspace(
    name: String,
    color: Option<String>,
) -> Result<crate::modules::workspaces::WorkspaceState, String> {
    crate::modules::workspaces::create_workspace(name, color)
}

#[tauri::command]
pub async fn update_workspace(
    id: String,
    name: String,
    color: Option<String>,
) -> Result<crate::modules::workspaces::WorkspaceState, String> {
    crate::modules::workspaces::update_workspace(id, name, color)
}

#[tauri::command]
pub async fn delete_workspace(
    id: String,
) -> Result<crate::modules::workspaces::WorkspaceState, String> {
    crate::modules::workspaces::delete_workspace(id)
}

#[tauri::command]
pub async fn set_active_workspace(
    id: String,
) -> Result<crate::modules::workspaces::WorkspaceState, String> {
    crate::modules::workspaces::set_active_workspace(id)
}

#[tauri::command]
pub async fn add_service(
    service: crate::modules::config::ServiceInput,
) -> Result<crate::modules::config::Service, String> {
    let mut config = load_config().map_err(|e| e.to_string())?;
    let workspace_id = resolve_workspace_id(&config, service.workspace_id)?;

    let service = crate::modules::config::Service {
        id: Uuid::new_v4().to_string(),
        workspace_id,
        name: service.name,
        category: service.category,
        preset: service.preset,
        cwd: service.cwd,
        command: service.command,
        args: service.args,
        setup_command: service.setup_command,
        env: service.env,
        url: service.url,
        port: service.port,
        color: service.color,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    config.services.push(service.clone());
    save_config(&config).map_err(|e| e.to_string())?;

    Ok(service)
}

#[tauri::command]
pub async fn update_service(
    id: String,
    service: crate::modules::config::ServiceInput,
) -> Result<crate::modules::config::Service, String> {
    let mut config = load_config().map_err(|e| e.to_string())?;
    let requested_workspace = service
        .workspace_id
        .clone()
        .filter(|workspace_id| !workspace_id.trim().is_empty());

    if let Some(idx) = config.services.iter().position(|s| s.id == id) {
        let mut updated = config.services[idx].clone();

        if let Some(workspace_id) = requested_workspace {
            if !config
                .workspaces
                .iter()
                .any(|workspace| workspace.id == workspace_id)
            {
                return Err("Workspace no encontrado".to_string());
            }
            updated.workspace_id = workspace_id;
        }

        updated.name = service.name;
        updated.category = service.category;
        updated.preset = service.preset;
        updated.cwd = service.cwd;
        updated.command = service.command;
        updated.args = service.args;
        updated.setup_command = service.setup_command;
        updated.env = service.env;
        updated.url = service.url;
        updated.port = service.port;
        updated.color = service.color;
        updated.updated_at = chrono::Utc::now();

        config.services[idx] = updated.clone();
        crate::modules::config::save_config(&config).map_err(|e| e.to_string())?;
        Ok(updated)
    } else {
        Err("Service not found".into())
    }
}

fn move_service_in_config(
    config: &mut crate::modules::config::Config,
    id: &str,
    workspace_id: &str,
) -> Result<crate::modules::config::Service, String> {
    if !config
        .workspaces
        .iter()
        .any(|workspace| workspace.id == workspace_id)
    {
        return Err("Workspace no encontrado".to_string());
    }

    let index = config
        .services
        .iter()
        .position(|service| service.id == id)
        .ok_or_else(|| "Service not found".to_string())?;

    if config.services[index].workspace_id == workspace_id {
        return Err("El servicio ya pertenece a ese workspace".to_string());
    }

    let mut service = config.services.remove(index);
    service.workspace_id = workspace_id.to_string();
    service.updated_at = chrono::Utc::now();
    config.services.push(service.clone());
    Ok(service)
}

fn reorder_services_in_config(
    config: &mut crate::modules::config::Config,
    workspace_id: &str,
    ordered_ids: &[String],
) -> Result<(), String> {
    if !config
        .workspaces
        .iter()
        .any(|workspace| workspace.id == workspace_id)
    {
        return Err("Workspace no encontrado".to_string());
    }

    let positions: Vec<usize> = config
        .services
        .iter()
        .enumerate()
        .filter(|(_, service)| service.workspace_id == workspace_id)
        .map(|(index, _)| index)
        .collect();

    let mut workspace_services: HashMap<String, crate::modules::config::Service> = config
        .services
        .iter()
        .filter(|service| service.workspace_id == workspace_id)
        .map(|service| (service.id.clone(), service.clone()))
        .collect();

    if workspace_services.len() != ordered_ids.len() {
        return Err("La lista de servicios no coincide con el workspace".to_string());
    }

    let mut ordered_services = Vec::with_capacity(ordered_ids.len());
    for id in ordered_ids {
        let service = workspace_services
            .remove(id)
            .ok_or_else(|| format!("Servicio duplicado o fuera del workspace: {id}"))?;
        ordered_services.push(service);
    }

    if !workspace_services.is_empty() {
        return Err("La lista no incluye todos los servicios del workspace".to_string());
    }

    for (position, service) in positions.into_iter().zip(ordered_services) {
        config.services[position] = service;
    }

    Ok(())
}

#[tauri::command]
pub async fn move_service(
    id: String,
    workspace_id: String,
) -> Result<crate::modules::config::Service, String> {
    let mut config = load_config().map_err(|error| error.to_string())?;
    let service_name = config
        .services
        .iter()
        .find(|service| service.id == id)
        .map(|service| service.name.clone())
        .ok_or_else(|| "Service not found".to_string())?;
    let from_workspace_id = config
        .services
        .iter()
        .find(|service| service.id == id)
        .map(|service| service.workspace_id.clone())
        .unwrap_or_default();
    let from_workspace = config
        .workspaces
        .iter()
        .find(|workspace| workspace.id == from_workspace_id)
        .map(|workspace| workspace.name.clone())
        .unwrap_or_else(|| from_workspace_id.clone());
    let to_workspace = config
        .workspaces
        .iter()
        .find(|workspace| workspace.id == workspace_id)
        .map(|workspace| workspace.name.clone())
        .ok_or_else(|| "Workspace no encontrado".to_string())?;

    let service = move_service_in_config(&mut config, &id, &workspace_id)?;
    save_config(&config).map_err(|error| error.to_string())?;

    crate::modules::dashboard_log::record(
        DashboardLogInput::info(
            "backend",
            "service.move",
            format!("{service_name} movido de {from_workspace} a {to_workspace}"),
        )
        .with_service(&id, &service_name)
        .with_context(json!({
            "fromWorkspaceId": from_workspace_id,
            "fromWorkspaceName": from_workspace,
            "toWorkspaceId": workspace_id,
            "toWorkspaceName": to_workspace,
        })),
    );

    Ok(service)
}

#[tauri::command]
pub async fn reorder_services(
    workspace_id: String,
    ordered_ids: Vec<String>,
) -> Result<(), String> {
    let mut config = load_config().map_err(|error| error.to_string())?;
    reorder_services_in_config(&mut config, &workspace_id, &ordered_ids)?;
    save_config(&config).map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn remove_service(id: String) -> Result<(), String> {
    let mut config = load_config().map_err(|e| e.to_string())?;
    if !config.services.iter().any(|service| service.id == id) {
        return Err("Service not found".to_string());
    }

    crate::modules::workspaces::delete_service_assets(&id);
    config.services.retain(|service| service.id != id);
    save_config(&config).map_err(|e| e.to_string())?;
    crate::modules::health::remove(&id);
    Ok(())
}

#[tauri::command]
pub async fn start_service(id: String) -> Result<(), String> {
    let config = load_config().map_err(|e| e.to_string())?;
    let service = find_service(&config, &id).ok_or("Service not found")?;

    if crate::modules::process::is_service_alive(&id) {
        crate::modules::dashboard_log::record(
            DashboardLogInput::error(
                "backend",
                "service.start",
                format!("{} ya está en ejecución", service.name),
            )
            .with_service(&service.id, &service.name),
        );
        return Err("Service already running".into());
    }

    let env_vars: std::collections::HashMap<String, String> = service
        .env
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    crate::modules::dashboard_log::record(
        DashboardLogInput::info(
            "backend",
            "service.start",
            format!("Iniciando {}", service.name),
        )
        .with_service(&service.id, &service.name)
        .with_context(json!({
            "cwd": service.cwd,
            "command": service.command,
            "args": service.args,
        })),
    );

    let handle = match crate::modules::process::process_start_service(
        &id,
        &service.cwd,
        &service.command,
        &service.args,
        &env_vars,
    ) {
        Ok(handle) => handle,
        Err(error) => {
            crate::modules::dashboard_log::record(
                DashboardLogInput::error(
                    "backend",
                    "service.start",
                    format!("No se pudo iniciar {}", service.name),
                )
                .with_service(&service.id, &service.name)
                .with_details(error.to_string())
                .with_context(json!({
                    "cwd": service.cwd,
                    "command": service.command,
                    "args": service.args,
                })),
            );
            return Err(error.to_string());
        }
    };

    if let Err(error) = crate::modules::runtime::register_runtime(
        &id,
        handle.pid,
        &handle.run_id,
        &handle.log_path,
        handle.process_created_at,
    ) {
        let _ = crate::modules::process::process_stop_service(&id);
        crate::modules::dashboard_log::record(
            DashboardLogInput::error(
                "backend",
                "service.start",
                format!("No se pudo registrar el runtime de {}", service.name),
            )
            .with_service(&service.id, &service.name)
            .with_details(error.to_string()),
        );
        return Err(error.to_string());
    }

    crate::modules::health::reset(&id, "checking");

    crate::modules::dashboard_log::record(
        DashboardLogInput::info(
            "backend",
            "service.start",
            format!("{} iniciado", service.name),
        )
        .with_service(&service.id, &service.name)
        .with_context(json!({
            "pid": handle.pid,
            "runId": handle.run_id,
            "logPath": handle.log_path,
        })),
    );

    Ok(())
}

#[tauri::command]
pub async fn stop_service(id: String) -> Result<(), String> {
    let config = load_config().ok();
    let service = config.as_ref().and_then(|config| find_service(config, &id));
    let service_name = service
        .map(|service| service.name.clone())
        .unwrap_or_else(|| id.clone());

    if let Err(error) = crate::modules::runtime::mark_service_stopping(&id) {
        crate::modules::dashboard_log::record(
            DashboardLogInput::error(
                "backend",
                "service.stop",
                format!("No se pudo preparar el cierre de {}", service_name),
            )
            .with_service(&id, &service_name)
            .with_details(error.to_string()),
        );
        return Err(error.to_string());
    }

    if let Err(error) = crate::modules::process::process_stop_service(&id) {
        crate::modules::dashboard_log::record(
            DashboardLogInput::error(
                "backend",
                "service.stop",
                format!("No se pudo detener {}", service_name),
            )
            .with_service(&id, &service_name)
            .with_details(error.to_string()),
        );
        return Err(error.to_string());
    }

    if let Err(error) = crate::modules::runtime::mark_service_stopped(&id) {
        crate::modules::dashboard_log::record(
            DashboardLogInput::error(
                "backend",
                "service.stop",
                format!("No se pudo actualizar el estado de {}", service_name),
            )
            .with_service(&id, &service_name)
            .with_details(error.to_string()),
        );
        return Err(error.to_string());
    }

    crate::modules::health::reset(&id, "stopped");

    crate::modules::dashboard_log::record(
        DashboardLogInput::info(
            "backend",
            "service.stop",
            format!("{} detenido", service_name),
        )
        .with_service(&id, &service_name),
    );

    if let Some(config) = config.as_ref() {
        let settings = &config.settings;
        if let Err(error) = crate::modules::logs::cleanup_old_logs(
            &id,
            settings.log_retention_runs,
            settings.log_retention_days as u64,
        )
        .await
        {
            crate::modules::dashboard_log::record(
                DashboardLogInput::warn(
                    "backend",
                    "logs.cleanup",
                    format!("No se pudieron limpiar los registros de {}", service_name),
                )
                .with_service(&id, &service_name)
                .with_details(error.to_string()),
            );
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn restart_service(id: String) -> Result<(), String> {
    let config = load_config().map_err(|e| e.to_string())?;
    let service = find_service(&config, &id).ok_or("Service not found")?;

    crate::modules::dashboard_log::record(
        DashboardLogInput::info(
            "backend",
            "service.restart",
            format!("Reiniciando {}", service.name),
        )
        .with_service(&service.id, &service.name),
    );

    for (stage, result) in [
        (
            "prepare",
            crate::modules::runtime::mark_service_stopping(&id).map_err(|error| error.to_string()),
        ),
        (
            "stop",
            crate::modules::process::process_stop_service(&id).map_err(|error| error.to_string()),
        ),
        (
            "state",
            crate::modules::runtime::mark_service_stopped(&id).map_err(|error| error.to_string()),
        ),
    ] {
        if let Err(error) = result {
            crate::modules::dashboard_log::record(
                DashboardLogInput::error(
                    "backend",
                    "service.restart",
                    format!("No se pudo reiniciar {} ({stage})", service.name),
                )
                .with_service(&service.id, &service.name)
                .with_details(error),
            );
            return Err("No se pudo reiniciar el servicio".into());
        }
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    let env_vars: std::collections::HashMap<String, String> = service
        .env
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let handle = match crate::modules::process::process_start_service(
        &id,
        &service.cwd,
        &service.command,
        &service.args,
        &env_vars,
    ) {
        Ok(handle) => handle,
        Err(error) => {
            crate::modules::dashboard_log::record(
                DashboardLogInput::error(
                    "backend",
                    "service.restart",
                    format!("No se pudo reiniciar {}", service.name),
                )
                .with_service(&service.id, &service.name)
                .with_details(error.to_string())
                .with_context(json!({
                    "cwd": service.cwd,
                    "command": service.command,
                    "args": service.args,
                })),
            );
            return Err(error.to_string());
        }
    };

    if let Err(error) = crate::modules::runtime::register_runtime(
        &id,
        handle.pid,
        &handle.run_id,
        &handle.log_path,
        handle.process_created_at,
    ) {
        let _ = crate::modules::process::process_stop_service(&id);
        crate::modules::dashboard_log::record(
            DashboardLogInput::error(
                "backend",
                "service.restart",
                format!("No se pudo registrar el runtime de {}", service.name),
            )
            .with_service(&service.id, &service.name)
            .with_details(error.to_string()),
        );
        return Err(error.to_string());
    }

    crate::modules::health::reset(&id, "checking");

    crate::modules::dashboard_log::record(
        DashboardLogInput::info(
            "backend",
            "service.restart",
            format!("{} reiniciado", service.name),
        )
        .with_service(&service.id, &service.name)
        .with_context(json!({
            "pid": handle.pid,
            "runId": handle.run_id,
            "logPath": handle.log_path,
        })),
    );

    Ok(())
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceStatusInfo {
    pub status: String,
    pub pid_alive: bool,
    pub exit_code: Option<i32>,
    pub last_error: Option<String>,
    pub health: Option<crate::modules::health::HealthStatus>,
}

#[tauri::command]
pub async fn get_service_statuses() -> Result<HashMap<String, ServiceStatusInfo>, String> {
    let config = load_config().map_err(|e| e.to_string())?;
    let runtime = crate::modules::config::load_runtime().unwrap_or_default();
    let mut statuses = HashMap::new();

    for service in &config.services {
        let entry = runtime.iter().find(|entry| entry.service_id == service.id);
        let alive = crate::modules::process::is_service_alive(&service.id);
        let explicitly_stopped = entry
            .map(|entry| entry.status == "stopping" || entry.status == "stopped")
            .unwrap_or(false);
        let fatal = if alive || entry.map(|entry| entry.status == "error").unwrap_or(false) {
            crate::modules::logs::detect_fatal_error(
                &service.id,
                entry.map(|entry| entry.run_id.as_str()),
            )
        } else {
            None
        };

        let status = if alive {
            if fatal.is_some() || entry.map(|entry| entry.status == "error").unwrap_or(false) {
                "error"
            } else if entry
                .map(|entry| entry.status == "running" || entry.status == "stopping")
                .unwrap_or(false)
            {
                "running"
            } else {
                "external"
            }
        } else if !explicitly_stopped
            && (fatal.is_some()
                || entry
                    .and_then(|entry| entry.exit_code)
                    .map(|code| code != 0)
                    .unwrap_or(false)
                || entry.map(|entry| entry.status == "error").unwrap_or(false))
        {
            "error"
        } else {
            "stopped"
        };

        let last_error = if status == "stopped" {
            None
        } else {
            fatal.or_else(|| entry.and_then(|entry| entry.last_error.clone()))
        };
        let health = if config.settings.health_check_enabled {
            crate::modules::health::snapshot(&service.id)
        } else {
            None
        };
        statuses.insert(
            service.id.clone(),
            ServiceStatusInfo {
                status: status.to_string(),
                pid_alive: alive,
                exit_code: entry.and_then(|entry| entry.exit_code),
                last_error,
                health,
            },
        );
    }

    Ok(statuses)
}

#[tauri::command]
pub async fn get_service_usage(
    ids: Vec<String>,
) -> Result<HashMap<String, crate::modules::metrics::ServiceUsage>, String> {
    tauri::async_runtime::spawn_blocking(move || crate::modules::metrics::collect(&ids))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn read_logs(
    id: String,
    run_id: Option<String>,
    lines: Option<usize>,
) -> Result<crate::modules::config::LogOutput, String> {
    crate::modules::logs::logs_read_logs(&id, run_id, lines)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_log_runs(id: String) -> Result<Vec<crate::modules::config::RunInfo>, String> {
    crate::modules::logs::list_runs(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_log_folder(id: String) -> Result<(), String> {
    crate::modules::logs::logs_open_log_folder(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_service_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn open_service_url(url: String) -> Result<(), String> {
    tauri_plugin_opener::open_url(&url, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_service_terminal(cwd: String) -> Result<(), String> {
    let path = std::path::PathBuf::from(cwd.trim());
    if !path.is_dir() {
        return Err(format!(
            "La carpeta de trabajo no existe: {}",
            path.to_string_lossy()
        ));
    }

    #[cfg(target_os = "windows")]
    {
        if which::which("wt").is_ok()
            && std::process::Command::new("wt")
                .arg("-d")
                .arg(&path)
                .spawn()
                .is_ok()
        {
            return Ok(());
        }

        std::process::Command::new("cmd")
            .args(["/K", "cd", "/d"])
            .arg(&path)
            .creation_flags(0x00000010)
            .spawn()
            .map(|_| ())
            .map_err(|error| {
                crate::modules::dashboard_log::record(
                    DashboardLogInput::error(
                        "backend",
                        "service.terminal",
                        "No se pudo abrir la terminal del servicio",
                    )
                    .with_details(error.to_string())
                    .with_context(json!({ "cwd": path })),
                );
                error.to_string()
            })
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Abrir terminal solo está disponible en Windows".to_string())
    }
}

#[tauri::command]
pub async fn open_service_editor(cwd: String) -> Result<(), String> {
    let path = std::path::PathBuf::from(cwd.trim());
    if !path.is_dir() {
        return Err(format!(
            "La carpeta de trabajo no existe: {}",
            path.to_string_lossy()
        ));
    }

    let editor = which::which("code")
        .or_else(|_| which::which("code.cmd"))
        .map_err(|_| "VS Code no está disponible en PATH".to_string())?;

    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("cmd")
        .arg("/C")
        .arg(&editor)
        .arg(&path)
        .creation_flags(0x08000000)
        .spawn();

    #[cfg(not(target_os = "windows"))]
    let result = std::process::Command::new(editor).arg(&path).spawn();

    result.map(|_| ()).map_err(|error| {
        crate::modules::dashboard_log::record(
            DashboardLogInput::error("backend", "service.editor", "No se pudo abrir VS Code")
                .with_details(error.to_string())
                .with_context(json!({ "cwd": path })),
        );
        error.to_string()
    })
}

#[tauri::command]
pub async fn diagnose_service(id: String) -> Result<Diagnostics, String> {
    let config = load_config().map_err(|e| e.to_string())?;
    let service = find_service(&config, &id).ok_or("Service not found")?;

    let mut warnings = Vec::new();
    let cwd = crate::modules::process::resolve_cwd(&service.cwd);

    // Check cwd exists
    let cwd_exists = cwd.is_dir();
    if !cwd_exists {
        warnings.push(format!("Carpeta de trabajo no existe: {}", cwd.display()));
    }

    // Check command exists
    let command_exists = if service.command.contains(' ') {
        // Complex command, check first part
        let parts: Vec<&str> = service.command.split_whitespace().collect();
        if let Some(cmd) = parts.first() {
            which::which(cmd).is_ok()
        } else {
            false
        }
    } else {
        which::which(&service.command).is_ok()
    };
    if !command_exists {
        warnings.push("Comando no encontrado en PATH".to_string());
    }

    // Check port
    let port_available = if let Some(port) = service.port {
        check_port_available(port)
    } else {
        true
    };
    if !port_available {
        warnings.push(format!("Puerto {} en uso", service.port.unwrap()));
    }

    // Check wrapper for Spring Boot
    let wrapper_found = if service.preset == "spring-boot" {
        cwd.join("mvnw.cmd").exists() || cwd.join("gradlew.bat").exists()
    } else {
        true
    };
    if !wrapper_found && service.preset == "spring-boot" {
        warnings.push("Wrapper de Maven/Gradle no encontrado".to_string());
    }

    Ok(Diagnostics {
        cwd_exists,
        command_exists,
        port_available,
        wrapper_found,
        warnings,
    })
}

fn check_port_available(port: u16) -> bool {
    use std::net::TcpListener;
    TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok()
}

#[tauri::command]
pub fn write_dashboard_log(
    entry: crate::modules::dashboard_log::DashboardLogInput,
) -> Result<crate::modules::dashboard_log::DashboardLogEntry, String> {
    crate::modules::dashboard_log::write(entry)
}

#[tauri::command]
pub fn read_dashboard_log(
    limit: Option<usize>,
    level: Option<String>,
    query: Option<String>,
) -> Result<crate::modules::dashboard_log::DashboardLogPage, String> {
    crate::modules::dashboard_log::read(limit, level, query)
}

#[tauri::command]
pub fn clear_dashboard_log() -> Result<(), String> {
    crate::modules::dashboard_log::clear()
}

#[tauri::command]
pub fn open_dashboard_log_folder() -> Result<(), String> {
    let folder = crate::modules::dashboard_log::folder()?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(folder)
            .spawn()
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    let config = load_config().map_err(|e| e.to_string())?;
    Ok(config.settings)
}

#[tauri::command]
pub async fn save_settings(settings: Settings) -> Result<(), String> {
    let mut config = load_config().map_err(|e| e.to_string())?;
    config.settings = settings;
    save_config(&config).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn shutdown_app(app: tauri::AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub async fn dialog_open(_path: String) -> Result<String, String> {
    // This will be handled by the dialog plugin
    Err("Use dialog plugin from frontend".into())
}

#[tauri::command]
pub async fn detect_preset(cwd: String) -> Result<Option<String>, String> {
    let path = std::path::Path::new(&cwd);

    if path.join("package.json").exists() {
        let content = std::fs::read_to_string(path.join("package.json")).unwrap_or_default();
        if content.contains("vue") || content.contains("@vue/cli") || content.contains("vite") {
            return Ok(Some("vue".to_string()));
        }
        return Ok(Some("custom".to_string()));
    }

    if path.join("mvnw.cmd").exists() || path.join("pom.xml").exists() {
        return Ok(Some("spring-boot".to_string()));
    }

    if path.join("gradlew.bat").exists() || path.join("build.gradle").exists() {
        return Ok(Some("spring-boot".to_string()));
    }

    Ok(None)
}

#[tauri::command]
pub async fn adopt_processes() -> Result<(), String> {
    crate::modules::runtime::adopt_runtime_processes().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reconcile_runtime() -> Result<(), String> {
    crate::modules::runtime::runtime_reconcile_runtime().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_log_stream(
    app: tauri::AppHandle,
    id: String,
    run_id: Option<String>,
) -> Result<(), String> {
    crate::modules::logs::start_log_stream(app, id, run_id)
}

#[tauri::command]
pub async fn stop_log_stream(id: String) -> Result<(), String> {
    crate::modules::logs::stop_log_stream(&id);
    Ok(())
}

#[tauri::command]
pub async fn start_setup_log_stream(app: tauri::AppHandle, task_id: String) -> Result<(), String> {
    crate::modules::environment::start_setup_log_stream(app, task_id)
}

#[tauri::command]
pub async fn stop_setup_log_stream(task_id: String) -> Result<(), String> {
    crate::modules::environment::stop_setup_log_stream(&task_id);
    Ok(())
}

#[tauri::command]
pub async fn detect_environment(
    cwd: String,
) -> Result<crate::modules::environment::EnvironmentInfo, String> {
    crate::modules::environment::detect_environment(&cwd)
}

#[tauri::command]
pub async fn start_setup(
    cwd: String,
    command: String,
) -> Result<crate::modules::environment::SetupStatus, String> {
    crate::modules::environment::start_setup(&cwd, &command)
}

#[tauri::command]
pub async fn get_setup_status(
    task_id: String,
) -> Result<crate::modules::environment::SetupStatus, String> {
    crate::modules::environment::get_setup_status(&task_id)
}

#[tauri::command]
pub async fn read_setup_log(
    task_id: String,
    lines: Option<usize>,
) -> Result<crate::modules::config::LogOutput, String> {
    crate::modules::environment::read_setup_log(&task_id, lines)
}

#[tauri::command]
pub async fn cancel_setup(task_id: String) -> Result<(), String> {
    crate::modules::environment::cancel_setup(&task_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::config::{Config, Service, Settings, Workspace, default_workspace};
    use std::collections::HashMap;

    fn test_service(id: &str, workspace_id: &str) -> Service {
        let now = chrono::Utc::now();
        Service {
            id: id.to_string(),
            workspace_id: workspace_id.to_string(),
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
    fn moves_service_between_workspaces() {
        let now = chrono::Utc::now();
        let destination = Workspace {
            id: "destination".to_string(),
            name: "Destino".to_string(),
            color: None,
            created_at: now,
            updated_at: now,
        };
        let mut config = Config {
            schema_version: 2,
            workspaces: vec![default_workspace(), destination],
            active_workspace_id: Some("default".to_string()),
            services: vec![test_service("service-1", "default")],
            settings: Settings::default(),
        };

        let moved = move_service_in_config(&mut config, "service-1", "destination")
            .expect("service should move");
        assert_eq!(moved.workspace_id, "destination");
        assert_eq!(config.services[0].workspace_id, "destination");
        assert!(move_service_in_config(&mut config, "service-1", "destination").is_err());
        assert!(move_service_in_config(&mut config, "service-1", "missing").is_err());
    }

    #[test]
    fn moves_service_to_end_of_destination() {
        let now = chrono::Utc::now();
        let destination = Workspace {
            id: "destination".to_string(),
            name: "Destino".to_string(),
            color: None,
            created_at: now,
            updated_at: now,
        };
        let mut config = Config {
            schema_version: 2,
            workspaces: vec![default_workspace(), destination],
            active_workspace_id: Some("default".to_string()),
            services: vec![
                test_service("a", "default"),
                test_service("b", "destination"),
                test_service("c", "default"),
                test_service("d", "destination"),
            ],
            settings: Settings::default(),
        };

        move_service_in_config(&mut config, "a", "destination").expect("service should move");

        let ids: Vec<&str> = config
            .services
            .iter()
            .map(|service| service.id.as_str())
            .collect();
        assert_eq!(ids, vec!["b", "c", "d", "a"]);
        let destination_ids: Vec<&str> = config
            .services
            .iter()
            .filter(|service| service.workspace_id == "destination")
            .map(|service| service.id.as_str())
            .collect();
        assert_eq!(destination_ids, vec!["b", "d", "a"]);
    }

    #[test]
    fn reorders_services_within_workspace() {
        let now = chrono::Utc::now();
        let destination = Workspace {
            id: "destination".to_string(),
            name: "Destino".to_string(),
            color: None,
            created_at: now,
            updated_at: now,
        };
        let mut config = Config {
            schema_version: 2,
            workspaces: vec![default_workspace(), destination],
            active_workspace_id: Some("default".to_string()),
            services: vec![
                test_service("a", "default"),
                test_service("b", "destination"),
                test_service("c", "default"),
                test_service("d", "destination"),
            ],
            settings: Settings::default(),
        };

        reorder_services_in_config(&mut config, "default", &["c".to_string(), "a".to_string()])
            .expect("services should reorder");

        let ids: Vec<&str> = config
            .services
            .iter()
            .map(|service| service.id.as_str())
            .collect();
        assert_eq!(ids, vec!["c", "b", "a", "d"]);
        assert!(reorder_services_in_config(&mut config, "default", &["c".to_string()]).is_err());
        assert!(
            reorder_services_in_config(&mut config, "default", &["c".to_string(), "b".to_string()])
                .is_err()
        );
        assert!(
            reorder_services_in_config(&mut config, "default", &["c".to_string(), "c".to_string()])
                .is_err()
        );
        assert!(
            reorder_services_in_config(&mut config, "missing", &["c".to_string(), "a".to_string()])
                .is_err()
        );
    }
}
