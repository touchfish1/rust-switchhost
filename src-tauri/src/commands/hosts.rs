use crate::hosts;
use crate::validation::validate_hosts_content;
use serde::Serialize;
use std::process::Command;

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
pub fn flush_dns_cache() -> Result<DnsFlushResult, String> {
    let platform = std::env::consts::OS.to_string();

    #[cfg(target_os = "windows")]
    {
        return run_dns_flush_command(
            "ipconfig",
            &["/flushdns"],
            &platform,
            "DNS 缓存已刷新",
            "刷新 DNS 缓存失败，请尝试以管理员身份运行应用后重试",
        );
    }

    #[cfg(target_os = "macos")]
    {
        run_simple_command("dscacheutil", &["-flushcache"])?;
        run_simple_command("killall", &["-HUP", "mDNSResponder"])?;
        return Ok(DnsFlushResult {
            success: true,
            platform,
            message: "DNS 缓存已刷新".to_string(),
        });
    }

    #[cfg(target_os = "linux")]
    {
        let candidates: [(&str, &[&str]); 4] = [
            ("resolvectl", &["flush-caches"]),
            ("systemd-resolve", &["--flush-caches"]),
            ("nscd", &["-i", "hosts"]),
            ("service", &["nscd", "restart"]),
        ];

        for (program, args) in candidates {
            if let Ok(result) = run_dns_flush_command(
                program,
                args,
                &platform,
                "DNS 缓存已刷新",
                "刷新 DNS 缓存失败，请尝试使用 sudo 或 root 权限运行应用后重试",
            ) {
                return Ok(result);
            }
        }

        return Ok(DnsFlushResult {
            success: false,
            platform,
            message: "当前系统未检测到可用的 DNS 刷新命令，请手动执行 resolvectl flush-caches 或 systemd-resolve --flush-caches".to_string(),
        });
    }

    #[allow(unreachable_code)]
    Ok(DnsFlushResult {
        success: false,
        platform,
        message: "当前平台暂不支持自动刷新 DNS 缓存".to_string(),
    })
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

fn run_simple_command(program: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("执行 {} 失败: {}", program, e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Err(if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            format!("命令 {} 执行失败", program)
        })
    }
}

fn run_dns_flush_command(
    program: &str,
    args: &[&str],
    platform: &str,
    success_message: &str,
    failure_hint: &str,
) -> Result<DnsFlushResult, String> {
    match run_simple_command(program, args) {
        Ok(()) => Ok(DnsFlushResult {
            success: true,
            platform: platform.to_string(),
            message: success_message.to_string(),
        }),
        Err(error) => Err(format!("{}: {}", failure_hint, error)),
    }
}
