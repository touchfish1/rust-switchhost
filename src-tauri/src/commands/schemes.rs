use crate::schemes::{Scheme, SchemeManager};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub scheme_manager: Mutex<SchemeManager>,
}

#[tauri::command]
pub fn get_all_schemes(state: State<AppState>) -> Result<Vec<Scheme>, String> {
    let manager = state.scheme_manager.lock().map_err(|e| e.to_string())?;
    manager.get_all_schemes().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_scheme(
    state: State<AppState>,
    name: String,
    content: String,
) -> Result<Scheme, String> {
    let mut manager = state.scheme_manager.lock().map_err(|e| e.to_string())?;
    manager
        .create_scheme(name, content)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_scheme(
    state: State<AppState>,
    id: String,
    name: String,
    content: String,
) -> Result<Scheme, String> {
    let mut manager = state.scheme_manager.lock().map_err(|e| e.to_string())?;
    manager
        .update_scheme(&id, name, content)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_scheme(state: State<AppState>, id: String) -> Result<(), String> {
    let mut manager = state.scheme_manager.lock().map_err(|e| e.to_string())?;
    manager.delete_scheme(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn switch_scheme(state: State<AppState>, id: String) -> Result<(), String> {
    let mut manager = state.scheme_manager.lock().map_err(|e| e.to_string())?;
    manager.switch_scheme(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_scheme_enabled(
    state: State<AppState>,
    id: String,
    enabled: bool,
) -> Result<Vec<Scheme>, String> {
    let mut manager = state.scheme_manager.lock().map_err(|e| e.to_string())?;
    manager
        .set_scheme_enabled(&id, enabled)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_schemes(state: State<AppState>, path: String) -> Result<(), String> {
    let manager = state.scheme_manager.lock().map_err(|e| e.to_string())?;
    manager
        .export_schemes(PathBuf::from(path))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_scheme_remote_config(
    state: State<AppState>,
    id: String,
    remote_url: Option<String>,
    auto_sync_enabled: bool,
    sync_interval_minutes: Option<u64>,
) -> Result<Scheme, String> {
    let mut manager = state.scheme_manager.lock().map_err(|e| e.to_string())?;
    manager
        .update_scheme_remote_config(&id, remote_url, auto_sync_enabled, sync_interval_minutes)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_schemes(state: State<AppState>, path: String) -> Result<Vec<Scheme>, String> {
    let mut manager = state.scheme_manager.lock().map_err(|e| e.to_string())?;
    manager
        .import_schemes(PathBuf::from(path))
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_remote_scheme(state: State<'_, AppState>, id: String) -> Result<Scheme, String> {
    let scheme = {
        let manager = state.scheme_manager.lock().map_err(|e| e.to_string())?;
        manager.get_scheme(&id).map_err(|e| e.to_string())?
    };

    let remote_url = scheme
        .remote_url
        .clone()
        .ok_or_else(|| "当前分组未配置远程 URL".to_string())?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let result = async {
        let response = client
            .get(&remote_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch URL: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))
    }
    .await;

    let mut manager = state.scheme_manager.lock().map_err(|e| e.to_string())?;

    match result {
        Ok(content) => manager
            .apply_remote_scheme_content(&id, content)
            .map_err(|e| e.to_string()),
        Err(error_message) => {
            let _ = manager.mark_remote_sync_error(&id, error_message.clone());
            Err(error_message)
        }
    }
}

#[tauri::command]
pub async fn fetch_remote_hosts(url: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch URL: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let content = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    Ok(content)
}
