use crate::schemes::{Scheme, SchemeManager};
use crate::validation::{validate_hosts_content, validate_remote_url};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

pub struct AppState {
    pub scheme_manager: Mutex<SchemeManager>,
}

#[tauri::command]
pub fn get_scheme_sync_logs(state: State<AppState>, id: String) -> Result<Vec<crate::schemes::SyncLogEntry>, String> {
    let manager = state.scheme_manager.lock().map_err(|e| e.to_string())?;
    manager.get_scheme_sync_logs(&id).map_err(|e| e.to_string())
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
    if let Some(url) = remote_url.as_deref() {
        validate_remote_url(url)?;
    }

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
pub async fn sync_remote_scheme(
    app: AppHandle,
    id: String,
    trigger: Option<String>,
) -> Result<Scheme, String> {
    perform_remote_sync(&app, id, trigger.unwrap_or_else(|| "manual".to_string())).await
}

#[tauri::command]
pub async fn fetch_remote_hosts(url: String) -> Result<String, String> {
    validate_remote_url(&url)?;

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

    validate_hosts_content(&content)?;

    Ok(content)
}

pub async fn perform_remote_sync(
    app: &AppHandle,
    id: String,
    trigger: String,
) -> Result<Scheme, String> {
    let scheme = {
        let state = app.state::<AppState>();
        let mut manager = state.scheme_manager.lock().map_err(|e| e.to_string())?;
        let scheme = manager.get_scheme(&id).map_err(|e| e.to_string())?;
        let remote_url = scheme
            .remote_url
            .clone()
            .ok_or_else(|| "当前分组未配置远程 URL".to_string())?;
        validate_remote_url(&remote_url)?;
        let _ = manager.mark_remote_sync_started(&id, &trigger).map_err(|e| e.to_string())?;
        scheme
    };

    let remote_url = scheme
        .remote_url
        .clone()
        .ok_or_else(|| "当前分组未配置远程 URL".to_string())?;

    emit_schemes_changed(app);

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

        let content = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        validate_hosts_content(&content)?;
        Ok(content)
    }
    .await;

    let state = app.state::<AppState>();
    let mut manager = state.scheme_manager.lock().map_err(|e| e.to_string())?;

    let updated = match result {
        Ok(content) => manager
            .apply_remote_scheme_content_with_trigger(&id, content, &trigger)
            .map_err(|e| e.to_string()),
        Err(error_message) => {
            let _ = manager.mark_remote_sync_error_with_trigger(&id, error_message.clone(), &trigger);
            Err(error_message)
        }
    };

    emit_schemes_changed(app);
    updated
}

fn emit_schemes_changed(app: &AppHandle) {
    let _ = app.emit("schemes-changed", ());
}
