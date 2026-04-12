mod commands;
mod error;
mod hosts;
mod schemes;
mod tray;
mod validation;

use commands::schemes::AppState;
use schemes::SchemeManager;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let scheme_manager = SchemeManager::new().expect("Failed to initialize SchemeManager");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState {
            scheme_manager: Mutex::new(scheme_manager),
        })
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }

            let tray_controller = tray::setup_tray(&app.handle())?;
            tray::start_metrics_monitor(tray_controller);
            start_background_sync(app.handle().clone());

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                let _ = window.hide();
                api.prevent_close();
            }
            WindowEvent::Focused(false) if window.label() == "tray-metrics" => {
                let _ = window.hide();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::restart_app,
            commands::hosts::check_hosts_permission,
            commands::hosts::get_hosts_content,
            commands::hosts::write_hosts_content,
            commands::hosts::flush_dns_cache,
            commands::hosts::list_hosts_backups,
            commands::hosts::get_hosts_backup_content,
            commands::hosts::restore_hosts_backup,
            commands::hosts::resolve_domain,
            commands::schemes::get_all_schemes,
            commands::schemes::create_scheme,
            commands::schemes::update_scheme,
            commands::schemes::delete_scheme,
            commands::schemes::switch_scheme,
            commands::schemes::set_scheme_enabled,
            commands::schemes::export_schemes,
            commands::schemes::import_schemes,
            commands::schemes::update_scheme_remote_config,
            commands::schemes::get_scheme_sync_logs,
            commands::schemes::sync_remote_scheme,
            commands::schemes::fetch_remote_hosts,
            commands::updates::check_for_updates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn start_background_sync(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let (due_jobs, wait_duration) = {
                let state = app.state::<AppState>();
                let next_cycle = match state.scheme_manager.lock() {
                    Ok(manager) => (
                        manager.get_due_sync_jobs(),
                        manager.get_next_sync_wait_duration(),
                    ),
                    Err(_) => (Vec::new(), std::time::Duration::from_secs(30)),
                };
                next_cycle
            };

            for job in due_jobs {
                let _ = commands::schemes::perform_remote_sync(&app, job.id, job.trigger).await;
            }

            tokio::time::sleep(wait_duration).await;
        }
    });
}
