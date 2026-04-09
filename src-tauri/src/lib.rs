mod commands;
mod hosts;
mod schemes;
mod tray;

use commands::schemes::AppState;
use schemes::SchemeManager;
use std::sync::Mutex;
use tauri::{Manager, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let scheme_manager = SchemeManager::new().expect("Failed to initialize SchemeManager");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState {
            scheme_manager: Mutex::new(scheme_manager),
        })
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            tray::setup_tray(&app.handle())?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::restart_app,
            commands::hosts::get_hosts_content,
            commands::hosts::write_hosts_content,
            commands::schemes::get_all_schemes,
            commands::schemes::create_scheme,
            commands::schemes::update_scheme,
            commands::schemes::delete_scheme,
            commands::schemes::switch_scheme,
            commands::schemes::set_scheme_enabled,
            commands::schemes::fetch_remote_hosts,
            commands::updates::check_for_updates,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
