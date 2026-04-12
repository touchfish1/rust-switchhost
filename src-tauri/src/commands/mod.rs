pub mod hosts;
pub mod schemes;
pub mod updates;
use crate::tray::{load_saved_metrics_window_state, save_metrics_window_state, TrayMetricsWindowState};
use tauri::AppHandle;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Rust SwitchHost!", name)
}

#[tauri::command]
pub fn restart_app(app: AppHandle) {
    app.restart();
}

#[tauri::command]
pub fn get_tray_metrics_window_state() -> Result<Option<TrayMetricsWindowState>, String> {
    load_saved_metrics_window_state().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn set_tray_metrics_window_state(state: TrayMetricsWindowState) -> Result<(), String> {
    save_metrics_window_state(state).map_err(|error| error.to_string())
}
