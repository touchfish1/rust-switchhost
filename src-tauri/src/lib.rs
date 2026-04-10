mod commands;
mod hosts;
mod schemes;
mod tray;
mod validation;

use commands::schemes::AppState;
use schemes::SchemeManager;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, WindowEvent};

const BACKGROUND_SYNC_INTERVAL_SECS: u64 = 30;

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

            tray::setup_tray(&app.handle())?;
            start_background_sync(app.handle().clone());

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::restart_app,
            commands::hosts::check_hosts_permission,
            commands::hosts::get_hosts_content,
            commands::hosts::write_hosts_content,
            commands::hosts::flush_dns_cache,
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
            let due_jobs = {
                let state = app.state::<AppState>();
                let jobs = match state.scheme_manager.lock() {
                    Ok(manager) => manager.get_due_sync_jobs(),
                    Err(_) => Vec::new(),
                };
                jobs
            };

            for job in due_jobs {
                let _ = commands::schemes::perform_remote_sync(&app, job.id, job.trigger).await;
            }

            tokio::time::sleep(std::time::Duration::from_secs(BACKGROUND_SYNC_INTERVAL_SECS)).await;
        }
    });
}
