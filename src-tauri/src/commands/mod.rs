pub mod hosts;
pub mod schemes;
pub mod updates;
use tauri::AppHandle;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Rust SwitchHost!", name)
}

#[tauri::command]
pub fn restart_app(app: AppHandle) {
    app.restart();
}
