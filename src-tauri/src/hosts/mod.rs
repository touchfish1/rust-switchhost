use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::path::PathBuf;

#[cfg(target_os = "linux")]
use std::path::Path;

#[cfg(target_os = "linux")]
use std::process::Command;

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
        Ok(()) => Ok(()),
        Err(error) => {
            #[cfg(target_os = "linux")]
            {
                if error.kind() == io::ErrorKind::PermissionDenied {
                    return write_hosts_file_with_pkexec(content);
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
