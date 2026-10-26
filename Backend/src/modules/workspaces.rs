use std::time::Duration;

use serde::Serialize;
use uuid::Uuid;

use crate::modules::config::{Config, Workspace, find_workspace, load_config, save_config};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceState {
    pub workspaces: Vec<Workspace>,
    pub active_workspace_id: String,
}

fn state_from_config(config: &Config) -> WorkspaceState {
    WorkspaceState {
        workspaces: config.workspaces.clone(),
        active_workspace_id: config
            .active_workspace_id
            .clone()
            .or_else(|| {
                config
                    .workspaces
                    .first()
                    .map(|workspace| workspace.id.clone())
            })
            .unwrap_or_default(),
    }
}

fn validate_name(config: &Config, name: &str, except_id: Option<&str>) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("El nombre del workspace es obligatorio".to_string());
    }
    if name.chars().count() > 60 {
        return Err("El nombre del workspace es demasiado largo".to_string());
    }

    let duplicate = config.workspaces.iter().any(|workspace| {
        workspace.id != except_id.unwrap_or_default() && workspace.name.eq_ignore_ascii_case(name)
    });
    if duplicate {
        return Err("Ya existe un workspace con ese nombre".to_string());
    }

    Ok(name.to_string())
}

fn normalize_color(color: Option<String>) -> Option<String> {
    let value = color.unwrap_or_default().trim().to_string();
    if value.is_empty() {
        return Some("#4f46e5".to_string());
    }
    if value.starts_with('#') && (value.len() == 4 || value.len() == 7) {
        Some(value)
    } else {
        Some("#4f46e5".to_string())
    }
}

pub fn get_workspace_state() -> Result<WorkspaceState, String> {
    let config = load_config().map_err(|error| error.to_string())?;
    Ok(state_from_config(&config))
}

pub fn create_workspace(name: String, color: Option<String>) -> Result<WorkspaceState, String> {
    let mut config = load_config().map_err(|error| error.to_string())?;
    let name = validate_name(&config, &name, None)?;
    let now = chrono::Utc::now();
    let workspace = Workspace {
        id: Uuid::new_v4().to_string(),
        name,
        color: normalize_color(color),
        created_at: now,
        updated_at: now,
    };

    config.workspaces.push(workspace.clone());
    config.active_workspace_id = Some(workspace.id);
    save_config(&config).map_err(|error| error.to_string())?;
    Ok(state_from_config(&config))
}

pub fn update_workspace(
    id: String,
    name: String,
    color: Option<String>,
) -> Result<WorkspaceState, String> {
    let mut config = load_config().map_err(|error| error.to_string())?;
    let name = validate_name(&config, &name, Some(&id))?;

    let workspace = config
        .workspaces
        .iter_mut()
        .find(|workspace| workspace.id == id)
        .ok_or_else(|| "Workspace no encontrado".to_string())?;

    workspace.name = name;
    workspace.color = normalize_color(color);
    workspace.updated_at = chrono::Utc::now();

    save_config(&config).map_err(|error| error.to_string())?;
    Ok(state_from_config(&config))
}

pub fn set_active_workspace(id: String) -> Result<WorkspaceState, String> {
    let mut config = load_config().map_err(|error| error.to_string())?;
    if find_workspace(&config, &id).is_none() {
        return Err("Workspace no encontrado".to_string());
    }
    config.active_workspace_id = Some(id);
    save_config(&config).map_err(|error| error.to_string())?;
    Ok(state_from_config(&config))
}

fn remove_service_assets(service_id: &str) {
    let _ = crate::modules::runtime::mark_service_stopping(service_id);
    let _ = crate::modules::process::process_stop_service(service_id);
    let _ = crate::modules::runtime::mark_service_stopped(service_id);
    let _ = crate::modules::runtime::unregister_runtime(service_id);

    if let Ok(log_dir) = crate::modules::config::service_log_dir(service_id) {
        let _ = std::fs::remove_dir_all(log_dir);
    }
}

pub fn delete_service_assets(service_id: &str) {
    crate::modules::logs::stop_log_stream(service_id);
    std::thread::sleep(Duration::from_millis(250));
    remove_service_assets(service_id);
}

pub fn delete_workspace(id: String) -> Result<WorkspaceState, String> {
    let mut config = load_config().map_err(|error| error.to_string())?;

    let workspace = config
        .workspaces
        .iter()
        .find(|workspace| workspace.id == id)
        .ok_or_else(|| "Workspace no encontrado".to_string())?;

    if config.workspaces.len() <= 1 {
        return Err("No se puede eliminar el último workspace".to_string());
    }

    let workspace_name = workspace.name.clone();
    let service_ids: Vec<String> = config
        .services
        .iter()
        .filter(|service| service.workspace_id == id)
        .map(|service| service.id.clone())
        .collect();

    for service_id in &service_ids {
        crate::modules::logs::stop_log_stream(service_id);
    }
    std::thread::sleep(Duration::from_millis(250));

    for service_id in &service_ids {
        remove_service_assets(service_id);
    }

    config.services.retain(|service| service.workspace_id != id);
    config.workspaces.retain(|workspace| workspace.id != id);

    if config.active_workspace_id.as_deref() == Some(&id) {
        config.active_workspace_id = config
            .workspaces
            .first()
            .map(|workspace| workspace.id.clone());
    }

    if config.active_workspace_id.is_none() {
        config.active_workspace_id = config
            .workspaces
            .first()
            .map(|workspace| workspace.id.clone());
    }

    save_config(&config).map_err(|error| error.to_string())?;

    eprintln!(
        "Workspace '{}' eliminado junto con {} servicio(s)",
        workspace_name,
        service_ids.len()
    );

    Ok(state_from_config(&config))
}
