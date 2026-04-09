use std::fs;
use std::io;
use std::fs::OpenOptions;
use std::path::PathBuf;

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

    fs::write(path, content)
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
