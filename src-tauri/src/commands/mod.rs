pub mod hosts;
pub mod schemes;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Rust SwitchHost!", name)
}
