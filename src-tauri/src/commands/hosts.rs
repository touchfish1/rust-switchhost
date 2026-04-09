use crate::hosts;

#[tauri::command]
pub fn get_hosts_content() -> Result<String, String> {
    hosts::read_hosts_file().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_hosts_content(content: String) -> Result<(), String> {
    hosts::write_hosts_file(&content).map_err(|e| e.to_string())
}
