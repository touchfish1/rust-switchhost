use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::path::PathBuf;
use std::process::Command;

#[cfg(target_os = "linux")]
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize)]
pub struct HostsBackupEntry {
    pub filename: String,
    pub path: String,
    pub created_at: String,
    pub size_bytes: u64,
    pub line_count: u64,
    pub host_entry_count: u64,
    pub comment_count: u64,
}

pub fn get_hosts_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
    }

    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("/etc/hosts")
    }
}

pub fn get_backup_dir() -> PathBuf {
    let config_dir = dirs::config_dir()
        .expect("Failed to get config directory")
        .join("rust-switchhost")
        .join("backups");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("Failed to create backup directory");
    }

    config_dir
}

pub fn read_hosts_file() -> io::Result<String> {
    let path = get_hosts_path();
    fs::read_to_string(path)
}

pub fn can_write_hosts_file() -> io::Result<()> {
    let path = get_hosts_path();
    OpenOptions::new().read(true).write(true).open(path).map(|_| ())
}

pub fn write_hosts_file(content: &str) -> io::Result<()> {
    let path = get_hosts_path();

    backup_hosts_file()?;

    match fs::write(&path, content) {
        Ok(()) => {
            if let Err(error) = flush_dns_cache() {
                eprintln!(
                    "Warning: Failed to flush DNS cache after writing hosts: {}",
                    error
                );
            }
            Ok(())
        }
        Err(error) => {
            #[cfg(target_os = "linux")]
            {
                if error.kind() == io::ErrorKind::PermissionDenied {
                    let result = write_hosts_file_with_pkexec(content);
                    if result.is_ok() {
                        if let Err(flush_error) = flush_dns_cache() {
                            eprintln!(
                                "Warning: Failed to flush DNS cache after pkexec write: {}",
                                flush_error
                            );
                        }
                    }
                    return result;
                }
            }

            Err(error)
        }
    }
}

pub fn backup_hosts_file() -> io::Result<String> {
    let hosts_path = get_hosts_path();
    let backup_dir = get_backup_dir();

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_filename = format!("hosts_{}.bak", timestamp);
    let backup_path = backup_dir.join(backup_filename);

    fs::copy(&hosts_path, &backup_path)?;

    Ok(backup_path.to_string_lossy().to_string())
}

pub fn list_backup_files() -> io::Result<Vec<HostsBackupEntry>> {
    let backup_dir = get_backup_dir();
    if !backup_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();

    for entry in fs::read_dir(&backup_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(extension) = path.extension() else {
            continue;
        };

        if extension != "bak" {
            continue;
        }

        let metadata = entry.metadata()?;
        let modified = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let created_at = chrono::DateTime::<chrono::Utc>::from(modified).to_rfc3339();

        let content = fs::read_to_string(&path).unwrap_or_default();
        let summary = summarize_hosts_content(&content);

        backups.push(HostsBackupEntry {
            filename: path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown.bak".to_string()),
            path: path.to_string_lossy().to_string(),
            created_at,
            size_bytes: metadata.len(),
            line_count: summary.line_count,
            host_entry_count: summary.host_entry_count,
            comment_count: summary.comment_count,
        });
    }

    backups.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(backups)
}

pub fn read_backup_file(path: &str) -> io::Result<String> {
    let backup_path = resolve_backup_path(path)?;
    fs::read_to_string(backup_path)
}

pub fn restore_backup_file(path: &str) -> io::Result<()> {
    let content = read_backup_file(path)?;
    write_hosts_file(&content)
}

pub fn flush_dns_cache() -> io::Result<String> {
    #[cfg(target_os = "windows")]
    {
        run_command("ipconfig", &["/flushdns"])?;
        return Ok("DNS 缓存已刷新".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        run_command("dscacheutil", &["-flushcache"])?;
        run_command("killall", &["-HUP", "mDNSResponder"])?;
        return Ok("DNS 缓存已刷新".to_string());
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
            if run_command(program, args).is_ok() {
                return Ok("DNS 缓存已刷新".to_string());
            }
        }

        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "当前系统未检测到可用的 DNS 刷新命令",
        ));
    }

    #[allow(unreachable_code)]
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "当前平台暂不支持自动刷新 DNS 缓存",
    ))
}

#[cfg(target_os = "linux")]
fn write_hosts_file_with_pkexec(content: &str) -> io::Result<()> {
    ensure_pkexec_available()?;

    let temp_path = create_temp_hosts_file(content)?;

    let status = Command::new("pkexec")
        .arg("/usr/bin/install")
        .arg("-m")
        .arg("644")
        .arg(&temp_path)
        .arg(get_hosts_path())
        .status();

    let cleanup_result = fs::remove_file(&temp_path);

    match status {
        Ok(status) if status.success() => {
            cleanup_result?;
            Ok(())
        }
        Ok(status) => {
            let _ = cleanup_result;
            let message = match status.code() {
                Some(126) => "pkexec 无法执行 install，请确认系统已安装 polkit 和 coreutils".to_string(),
                Some(127) => "pkexec 未找到 /usr/bin/install，请确认系统环境完整".to_string(),
                Some(code) => format!("pkexec 提权写入 /etc/hosts 失败，退出码: {}", code),
                None => "pkexec 提权写入 /etc/hosts 被中断".to_string(),
            };
            Err(io::Error::new(io::ErrorKind::PermissionDenied, message))
        }
        Err(error) => {
            let _ = cleanup_result;
            Err(io::Error::new(
                error.kind(),
                format!("无法启动 pkexec。请确认系统已安装 polkit，并在桌面会话中运行应用: {}", error),
            ))
        }
    }
}

#[cfg(target_os = "linux")]
fn ensure_pkexec_available() -> io::Result<()> {
    let paths = [Path::new("/usr/bin/pkexec"), Path::new("/bin/pkexec")];
    if paths.iter().any(|path| path.exists()) {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "未找到 pkexec。请先安装 polkit，例如在 Ubuntu 上安装 policykit-1",
        ))
    }
}

#[cfg(target_os = "linux")]
fn create_temp_hosts_file(content: &str) -> io::Result<PathBuf> {
    let filename = format!("rust-switchhost-hosts-{}.tmp", uuid::Uuid::new_v4());
    let path = std::env::temp_dir().join(filename);
    fs::write(&path, content)?;
    Ok(path)
}

fn run_command(program: &str, args: &[&str]) -> io::Result<()> {
    let output = Command::new(program).args(args).output()?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let message = if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            format!("命令 {} 执行失败", program)
        };
        Err(io::Error::other(message))
    }
}

fn resolve_backup_path(path: &str) -> io::Result<PathBuf> {
    let candidate = PathBuf::from(path);
    let requested = fs::canonicalize(&candidate)?;
    let backup_dir = fs::canonicalize(get_backup_dir())?;

    if !requested.starts_with(&backup_dir) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "只能访问备份目录内的文件",
        ));
    }

    if requested.extension().and_then(|extension| extension.to_str()) != Some("bak") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "仅支持恢复 .bak 备份文件",
        ));
    }

    Ok(requested)
}

#[derive(Debug, Default, PartialEq, Eq)]
struct HostsContentSummary {
    line_count: u64,
    host_entry_count: u64,
    comment_count: u64,
}

fn summarize_hosts_content(content: &str) -> HostsContentSummary {
    let mut summary = HostsContentSummary::default();

    for line in content.lines() {
        summary.line_count += 1;

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('#') {
            summary.comment_count += 1;
            continue;
        }

        let segments: Vec<&str> = trimmed.split_whitespace().collect();
        if segments.len() >= 2 {
            summary.host_entry_count += 1;
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::summarize_hosts_content;

    #[test]
    fn summarizes_hosts_backup_content() {
        let summary = summarize_hosts_content(
            "# comment\n127.0.0.1 localhost\n\n192.168.1.1 intranet.local # inline\n# second\n",
        );

        assert_eq!(summary.line_count, 5);
        assert_eq!(summary.host_entry_count, 2);
        assert_eq!(summary.comment_count, 2);
    }
}
