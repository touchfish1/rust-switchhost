use crate::hosts;
use crate::validation::validate_hosts_content;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HostsPermissionInfo {
    pub has_permission: bool,
    pub hosts_path: String,
    pub platform: String,
    pub message: String,
}

#[tauri::command]
pub fn get_hosts_content() -> Result<String, String> {
    hosts::read_hosts_file().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn write_hosts_content(content: String) -> Result<(), String> {
    validate_hosts_content(&content)?;
    hosts::write_hosts_file(&content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn check_hosts_permission() -> HostsPermissionInfo {
    let platform = std::env::consts::OS.to_string();
    let hosts_path = hosts::get_hosts_path().to_string_lossy().to_string();
    let has_permission = hosts::can_write_hosts_file().is_ok();

    let message = if has_permission {
        "当前已具备修改 Hosts 文件的权限".to_string()
    } else {
        match platform.as_str() {
            "windows" => format!(
                "当前没有修改 Hosts 文件的权限。请关闭应用后以管理员身份重新运行，目标文件：{}",
                hosts_path
            ),
            "macos" => format!(
                "当前没有修改 Hosts 文件的权限。请使用管理员权限启动应用，或确保当前用户可写入 {}",
                hosts_path
            ),
            _ => format!(
                "当前没有修改 Hosts 文件的权限。不要直接用 sudo 启动图形界面应用，否则可能丢失桌面 session bus。请改为让当前桌面用户运行应用，并通过 polkit/pkexec 或系统权限配置来写入 {}",
                hosts_path
            ),
        }
    };

    HostsPermissionInfo {
        has_permission,
        hosts_path,
        platform,
        message,
    }
}
