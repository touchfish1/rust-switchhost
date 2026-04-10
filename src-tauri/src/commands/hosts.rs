use crate::error::{AppResult, IntoCommandResult};
use crate::hosts;
use crate::validation::validate_hosts_content;
use serde::Serialize;
use std::net::ToSocketAddrs;

#[derive(Debug, Serialize)]
pub struct HostsPermissionInfo {
    pub has_permission: bool,
    pub hosts_path: String,
    pub platform: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DnsFlushResult {
    pub success: bool,
    pub platform: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DnsLookupResult {
    pub domain: String,
    pub success: bool,
    pub addresses: Vec<String>,
    pub message: String,
}

#[tauri::command]
pub fn get_hosts_content() -> Result<String, String> {
    get_hosts_content_impl().into_command_result()
}

#[tauri::command]
pub fn write_hosts_content(content: String) -> Result<(), String> {
    write_hosts_content_impl(content).into_command_result()
}

#[tauri::command]
pub fn list_hosts_backups() -> Result<Vec<hosts::HostsBackupEntry>, String> {
    list_hosts_backups_impl().into_command_result()
}

#[tauri::command]
pub fn get_hosts_backup_content(path: String) -> Result<String, String> {
    get_hosts_backup_content_impl(path).into_command_result()
}

#[tauri::command]
pub fn restore_hosts_backup(path: String) -> Result<String, String> {
    restore_hosts_backup_impl(path).into_command_result()
}

#[tauri::command]
pub fn resolve_domain(domain: String) -> Result<DnsLookupResult, String> {
    resolve_domain_impl(domain).into_command_result()
}

#[tauri::command]
pub fn flush_dns_cache() -> Result<DnsFlushResult, String> {
    let platform = std::env::consts::OS.to_string();
    match hosts::flush_dns_cache() {
        Ok(message) => Ok(DnsFlushResult {
            success: true,
            platform,
            message,
        }),
        Err(error) => Ok(DnsFlushResult {
            success: false,
            platform,
            message: format!("刷新 DNS 缓存失败: {}", error),
        }),
    }
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

fn get_hosts_content_impl() -> AppResult<String> {
    Ok(hosts::read_hosts_file()?)
}

fn write_hosts_content_impl(content: String) -> AppResult<()> {
    validate_hosts_content(&content)?;
    Ok(hosts::write_hosts_file(&content)?)
}

fn list_hosts_backups_impl() -> AppResult<Vec<hosts::HostsBackupEntry>> {
    Ok(hosts::list_backup_files()?)
}

fn get_hosts_backup_content_impl(path: String) -> AppResult<String> {
    Ok(hosts::read_backup_file(&path)?)
}

fn restore_hosts_backup_impl(path: String) -> AppResult<String> {
    hosts::restore_backup_file(&path)?;
    Ok("已恢复所选备份并重新写入系统 Hosts".to_string())
}

fn resolve_domain_impl(domain: String) -> AppResult<DnsLookupResult> {
    let normalized = domain.trim().to_string();
    if normalized.is_empty() {
        return Ok(DnsLookupResult {
            domain: normalized,
            success: false,
            addresses: Vec::new(),
            message: "请输入要诊断的域名".to_string(),
        });
    }

    match (normalized.as_str(), 0).to_socket_addrs() {
        Ok(addresses) => {
            let mut resolved: Vec<String> = addresses.map(|address| address.ip().to_string()).collect();
            resolved.sort();
            resolved.dedup();

            Ok(DnsLookupResult {
                domain: normalized,
                success: !resolved.is_empty(),
                message: if resolved.is_empty() {
                    "未解析到任何 IP 地址".to_string()
                } else {
                    format!("共解析到 {} 个唯一 IP 地址", resolved.len())
                },
                addresses: resolved,
            })
        }
        Err(error) => Ok(DnsLookupResult {
            domain: normalized,
            success: false,
            addresses: Vec::new(),
            message: format!("解析失败: {}", error),
        }),
    }
}
