use crate::error::{AppError, AppResult, IntoCommandResult};
use crate::schemes::{Scheme, SchemeManager};
use crate::validation::{validate_hosts_content, validate_remote_url};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use tauri::{AppHandle, Emitter, Manager, State};

pub struct AppState {
    pub scheme_manager: Mutex<SchemeManager>,
}

#[tauri::command]
pub fn get_scheme_sync_logs(state: State<AppState>, id: String) -> Result<Vec<crate::schemes::SyncLogEntry>, String> {
    get_scheme_sync_logs_impl(state, id).into_command_result()
}

#[tauri::command]
pub fn get_all_schemes(state: State<AppState>) -> Result<Vec<Scheme>, String> {
    get_all_schemes_impl(state).into_command_result()
}

#[tauri::command]
pub fn create_scheme(
    state: State<AppState>,
    name: String,
    content: String,
) -> Result<Scheme, String> {
    create_scheme_impl(state, name, content).into_command_result()
}

#[tauri::command]
pub fn update_scheme(
    state: State<AppState>,
    id: String,
    name: String,
    content: String,
) -> Result<Scheme, String> {
    update_scheme_impl(state, id, name, content).into_command_result()
}

#[tauri::command]
pub fn delete_scheme(state: State<AppState>, id: String) -> Result<(), String> {
    delete_scheme_impl(state, id).into_command_result()
}

#[tauri::command]
pub fn switch_scheme(state: State<AppState>, id: String) -> Result<(), String> {
    switch_scheme_impl(state, id).into_command_result()
}

#[tauri::command]
pub fn set_scheme_enabled(
    state: State<AppState>,
    id: String,
    enabled: bool,
) -> Result<Vec<Scheme>, String> {
    set_scheme_enabled_impl(state, id, enabled).into_command_result()
}

#[tauri::command]
pub fn export_schemes(state: State<AppState>, path: String) -> Result<(), String> {
    export_schemes_impl(state, path).into_command_result()
}

#[tauri::command]
pub fn update_scheme_remote_config(
    state: State<AppState>,
    id: String,
    remote_url: Option<String>,
    auto_sync_enabled: bool,
    sync_interval_minutes: Option<u64>,
) -> Result<Scheme, String> {
    update_scheme_remote_config_impl(state, id, remote_url, auto_sync_enabled, sync_interval_minutes)
        .into_command_result()
}

#[tauri::command]
pub fn import_schemes(state: State<AppState>, path: String) -> Result<Vec<Scheme>, String> {
    import_schemes_impl(state, path).into_command_result()
}

#[tauri::command]
pub async fn sync_remote_scheme(
    app: AppHandle,
    id: String,
    trigger: Option<String>,
) -> Result<Scheme, String> {
    perform_remote_sync(&app, id, trigger.unwrap_or_else(|| "manual".to_string())).await
        .into_command_result()
}

#[tauri::command]
pub async fn fetch_remote_hosts(url: String) -> Result<String, String> {
    fetch_remote_hosts_impl(url).await.into_command_result()
}

pub async fn perform_remote_sync(
    app: &AppHandle,
    id: String,
    trigger: String,
) -> AppResult<Scheme> {
    let scheme = {
        let state = app.state::<AppState>();
        let mut manager = lock_manager(&state)?;
        let scheme = manager.get_scheme(&id).map_err(map_scheme_error)?;
        let remote_url = scheme
            .remote_url
            .clone()
            .ok_or_else(|| AppError::validation("当前分组未配置远程 URL"))?;
        validate_remote_url(&remote_url)?;
        let _ = manager
            .mark_remote_sync_started(&id, &trigger)
            .map_err(map_scheme_error)?;
        scheme
    };

    let remote_url = scheme
        .remote_url
        .clone()
        .ok_or_else(|| AppError::validation("当前分组未配置远程 URL"))?;

    emit_schemes_changed(app);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|error| AppError::network(format!("创建 HTTP 客户端失败: {}", error)))?;

    let result = async {
        let response = client
            .get(&remote_url)
            .send()
            .await
            .map_err(|error| AppError::network(format!("拉取远程内容失败: {}", error)))?;

        if !response.status().is_success() {
            return Err(AppError::network(format!("远程服务返回异常状态: {}", response.status())));
        }

        let content = response
            .text()
            .await
            .map_err(|error| AppError::network(format!("读取远程响应失败: {}", error)))?;

        validate_hosts_content(&content)?;
        Ok(content)
    }
    .await;

    let state = app.state::<AppState>();
    let mut manager = lock_manager(&state)?;

    let updated = match result {
        Ok(content) => manager
            .apply_remote_scheme_content_with_trigger(&id, content, &trigger)
            .map_err(map_scheme_error),
        Err(error_message) => {
            let _ = manager.mark_remote_sync_error_with_trigger(&id, error_message.to_string(), &trigger);
            Err(error_message)
        }
    };

    emit_schemes_changed(app);
    updated
}

fn emit_schemes_changed(app: &AppHandle) {
    let _ = app.emit("schemes-changed", ());
}

fn lock_manager<'a>(state: &'a State<AppState>) -> AppResult<MutexGuard<'a, SchemeManager>> {
    Ok(state.scheme_manager.lock()?)
}

fn get_scheme_sync_logs_impl(state: State<AppState>, id: String) -> AppResult<Vec<crate::schemes::SyncLogEntry>> {
    let manager = lock_manager(&state)?;
    Ok(manager.get_scheme_sync_logs(&id)?)
}

fn get_all_schemes_impl(state: State<AppState>) -> AppResult<Vec<Scheme>> {
    let manager = lock_manager(&state)?;
    Ok(manager.get_all_schemes()?)
}

fn create_scheme_impl(state: State<AppState>, name: String, content: String) -> AppResult<Scheme> {
    validate_hosts_content(&content)?;
    let mut manager = lock_manager(&state)?;
    Ok(manager.create_scheme(name, content).map_err(map_scheme_error)?)
}

fn update_scheme_impl(state: State<AppState>, id: String, name: String, content: String) -> AppResult<Scheme> {
    validate_hosts_content(&content)?;
    let mut manager = lock_manager(&state)?;
    Ok(manager.update_scheme(&id, name, content).map_err(map_scheme_error)?)
}

fn delete_scheme_impl(state: State<AppState>, id: String) -> AppResult<()> {
    let mut manager = lock_manager(&state)?;
    Ok(manager.delete_scheme(&id).map_err(map_scheme_error)?)
}

fn switch_scheme_impl(state: State<AppState>, id: String) -> AppResult<()> {
    let mut manager = lock_manager(&state)?;
    Ok(manager.switch_scheme(&id).map_err(map_scheme_error)?)
}

fn set_scheme_enabled_impl(state: State<AppState>, id: String, enabled: bool) -> AppResult<Vec<Scheme>> {
    let mut manager = lock_manager(&state)?;
    Ok(manager.set_scheme_enabled(&id, enabled).map_err(map_scheme_error)?)
}

fn export_schemes_impl(state: State<AppState>, path: String) -> AppResult<()> {
    let manager = lock_manager(&state)?;
    Ok(manager.export_schemes(PathBuf::from(path))?)
}

fn update_scheme_remote_config_impl(
    state: State<AppState>,
    id: String,
    remote_url: Option<String>,
    auto_sync_enabled: bool,
    sync_interval_minutes: Option<u64>,
) -> AppResult<Scheme> {
    if let Some(url) = remote_url.as_deref() {
        validate_remote_url(url)?;
    }

    let mut manager = lock_manager(&state)?;
    Ok(manager
        .update_scheme_remote_config(&id, remote_url, auto_sync_enabled, sync_interval_minutes)
        .map_err(map_scheme_error)?)
}

fn import_schemes_impl(state: State<AppState>, path: String) -> AppResult<Vec<Scheme>> {
    let mut manager = lock_manager(&state)?;
    Ok(manager.import_schemes(PathBuf::from(path))?)
}

async fn fetch_remote_hosts_impl(url: String) -> AppResult<String> {
    validate_remote_url(&url)?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|error| AppError::network(format!("创建 HTTP 客户端失败: {}", error)))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|error| AppError::network(format!("拉取远程内容失败: {}", error)))?;

    if !response.status().is_success() {
        return Err(AppError::network(format!("远程服务返回异常状态: {}", response.status())));
    }

    let content = response
        .text()
        .await
        .map_err(|error| AppError::network(format!("读取远程响应失败: {}", error)))?;

    validate_hosts_content(&content)?;
    Ok(content)
}

fn map_scheme_error(error: std::io::Error) -> AppError {
    if error.kind() == std::io::ErrorKind::NotFound {
        AppError::SchemeNotFound("分组不存在".to_string())
    } else {
        AppError::from(error)
    }
}
