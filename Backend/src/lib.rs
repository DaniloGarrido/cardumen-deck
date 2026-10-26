use tauri::Manager;

mod modules {
    pub mod commands;
    pub mod config;
    pub mod dashboard_log;
    pub mod dependencies;
    pub mod environment;
    pub mod git;
    pub mod health;
    pub mod logs;
    pub mod metrics;
    pub mod process;
    pub mod runtime;
    pub mod workspaces;
}

pub use modules::commands::*;

fn install_dashboard_logging() {
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        modules::dashboard_log::record(
            modules::dashboard_log::DashboardLogInput::error(
                "system",
                "panic",
                "La aplicación detectó un panic interno",
            )
            .with_details(info.to_string()),
        );
        previous_hook(info);
    }));

    modules::dashboard_log::record(
        modules::dashboard_log::DashboardLogInput::info(
            "system",
            "app.start",
            "Cardumen Deck iniciado",
        )
        .with_context(serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "os": std::env::consts::OS,
        })),
    );
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let migration = modules::config::migrate_legacy_data();
    install_dashboard_logging();

    match migration {
        Ok(true) => {
            modules::dashboard_log::record(modules::dashboard_log::DashboardLogInput::info(
                "system",
                "config.migrate",
                "Datos migrados desde ServiceDashboard",
            ))
        }
        Ok(false) => {}
        Err(error) => modules::dashboard_log::record(
            modules::dashboard_log::DashboardLogInput::warn(
                "system",
                "config.migrate",
                "No se pudieron migrar los datos anteriores",
            )
            .with_details(error.to_string()),
        ),
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            greet,
            get_services,
            get_git_branches,
            get_git_repositories,
            get_git_state,
            git_fetch,
            git_pull,
            git_checkout,
            git_create_branch,
            git_delete_branch,
            git_merge,
            git_abort_operation,
            git_push,
            get_service_dependencies,
            get_workspace_state,
            create_workspace,
            update_workspace,
            delete_workspace,
            set_active_workspace,
            add_service,
            update_service,
            move_service,
            reorder_services,
            remove_service,
            start_service,
            stop_service,
            restart_service,
            get_service_statuses,
            get_service_usage,
            read_logs,
            list_log_runs,
            open_log_folder,
            open_service_folder,
            open_service_url,
            open_service_terminal,
            open_service_editor,
            diagnose_service,
            detect_environment,
            start_log_stream,
            stop_log_stream,
            start_setup_log_stream,
            stop_setup_log_stream,
            get_settings,
            save_settings,
            write_dashboard_log,
            read_dashboard_log,
            clear_dashboard_log,
            open_dashboard_log_folder,
            shutdown_app,
            dialog_open,
            detect_preset,
            start_setup,
            get_setup_status,
            read_setup_log,
            cancel_setup,
            adopt_processes,
            reconcile_runtime,
        ])
        .setup(|app| {
            #[cfg(desktop)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.set_title("Cardumen Deck").unwrap();
                if let Some(icon) = app.default_window_icon().cloned() {
                    let _ = window.set_icon(icon);
                }
            }

            // Adopt processes on startup and reconcile stale runtime entries.
            let _ = modules::process::adopt_processes();
            let _ = modules::runtime::runtime_reconcile_runtime();
            modules::metrics::initialize();
            modules::health::start_monitor();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
