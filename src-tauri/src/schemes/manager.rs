use super::{Scheme, SchemeConfig, SyncLogEntry};
use crate::hosts;
use crate::validation::validate_hosts_content;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

pub struct SchemeManager {
    config: SchemeConfig,
    config_path: PathBuf,
    sync_log_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DueSyncJob {
    pub id: String,
    pub trigger: String,
}

const MIN_BACKGROUND_SYNC_WAIT_SECS: u64 = 5;
const MAX_BACKGROUND_SYNC_WAIT_SECS: u64 = 60;

impl SchemeManager {
    const MAX_SYNC_LOGS: usize = 50;

    pub fn new() -> io::Result<Self> {
        let config_dir = Self::get_config_dir()?;

        if !config_dir.exists() {
            if let Err(e) = fs::create_dir_all(&config_dir) {
                eprintln!(
                    "Warning: Failed to create config directory: {}, using in-memory config",
                    e
                );
                return Ok(Self {
                    config: SchemeConfig::default(),
                    config_path: config_dir.join("schemes.json"),
                    sync_log_path: config_dir.join("sync-logs.json"),
                });
            }
        }

        let config_path = config_dir.join("schemes.json");
        let sync_log_path = config_dir.join("sync-logs.json");

        let config = if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to read config file: {}, using default config",
                        e
                    );
                    SchemeConfig::default()
                }
            }
        } else {
            SchemeConfig::default()
        };

        let mut manager = Self {
            config,
            config_path,
            sync_log_path,
        };
        manager.migrate_embedded_logs_to_store();
        manager.migrate_legacy_platform_state();
        manager.reset_sync_runtime_state();
        Ok(manager)
    }

    pub fn get_all_schemes(&self) -> io::Result<Vec<Scheme>> {
        let current_platform = Self::current_platform();
        let active_ids = self.active_ids_for_platform(&current_platform);

        Ok(self
            .config
            .schemes
            .iter()
            .cloned()
            .map(|mut scheme| {
                scheme.enabled = active_ids.iter().any(|active_id| active_id == &scheme.id);
                scheme
            })
            .collect())
    }

    pub fn get_scheme_sync_logs(&self, id: &str) -> io::Result<Vec<SyncLogEntry>> {
        Ok(Self::load_sync_logs_from(&self.sync_log_path)?
            .remove(id)
            .unwrap_or_default())
    }

    pub fn get_due_sync_jobs(&self) -> Vec<DueSyncJob> {
        let now = Utc::now();

        self.config
            .schemes
            .iter()
            .filter_map(|scheme| {
                let remote_url = scheme.remote_url.as_ref()?.trim();
                let interval = scheme.sync_interval_minutes?;
                if !scheme.auto_sync_enabled || remote_url.is_empty() || interval == 0 {
                    return None;
                }

                if scheme.sync_status == "syncing" {
                    return None;
                }

                if let Some(next_retry_at) = scheme.next_retry_at {
                    return if next_retry_at <= now {
                        Some(DueSyncJob {
                            id: scheme.id.clone(),
                            trigger: "retry".to_string(),
                        })
                    } else {
                        None
                    };
                }

                let last_synced_at = scheme
                    .last_synced_at
                    .unwrap_or_else(|| DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH));

                let due_at = last_synced_at + Duration::minutes(interval as i64);
                if due_at <= now {
                    return Some(DueSyncJob {
                        id: scheme.id.clone(),
                        trigger: "scheduled".to_string(),
                    });
                }

                None
            })
            .collect()
    }

    pub fn get_next_sync_wait_duration(&self) -> std::time::Duration {
        self.get_next_sync_wait_duration_from(Utc::now())
    }

    pub fn create_scheme(&mut self, name: String, content: String) -> io::Result<Scheme> {
        let normalized_name = self.validate_scheme_name(&name, None)?;
        let scheme = Scheme::new(normalized_name, content);
        self.config.schemes.push(scheme.clone());
        self.save_config()?;
        Ok(scheme)
    }

    pub fn update_scheme(&mut self, id: &str, name: String, content: String) -> io::Result<Scheme> {
        let normalized_name = self.validate_scheme_name(&name, Some(id))?;
        if let Some(scheme) = self.config.schemes.iter_mut().find(|s| s.id == id) {
            scheme.name = normalized_name;
            scheme.content = content;
            scheme.updated_at = Utc::now();
            let updated = scheme.clone();
            self.save_config()?;
            Ok(updated)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "Scheme not found"))
        }
    }

    pub fn get_scheme(&self, id: &str) -> io::Result<Scheme> {
        self.config
            .schemes
            .iter()
            .find(|scheme| scheme.id == id)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Scheme not found"))
    }

    pub fn delete_scheme(&mut self, id: &str) -> io::Result<()> {
        let was_enabled = self
            .config
            .active_scheme_ids_by_platform
            .get(&Self::current_platform())
            .map(|ids| ids.iter().any(|scheme_id| scheme_id == id))
            .unwrap_or(false);

        self.config.schemes.retain(|s| s.id != id);
        for ids in self.config.active_scheme_ids_by_platform.values_mut() {
            ids.retain(|scheme_id| scheme_id != id);
        }

        if was_enabled {
            self.apply_active_schemes_for_current_platform()?;
        }

        self.sync_enabled_flags();
        self.save_config()
    }

    pub fn switch_scheme(&mut self, id: &str) -> io::Result<()> {
        if !self.config.schemes.iter().any(|scheme| scheme.id == id) {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Scheme not found"));
        }

        let platform = Self::current_platform();
        let mut active_ids = self.active_ids_for_platform(&platform);
        if !active_ids.iter().any(|active_id| active_id == id) {
            active_ids.push(id.to_string());
        }

        self.config
            .active_scheme_ids_by_platform
            .insert(platform, active_ids);
        self.apply_active_schemes_for_current_platform()?;
        self.sync_enabled_flags();
        self.save_config()
    }

    pub fn set_scheme_enabled(&mut self, id: &str, enabled: bool) -> io::Result<Vec<Scheme>> {
        if !self.config.schemes.iter().any(|scheme| scheme.id == id) {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Scheme not found"));
        }

        if enabled {
            let platform = Self::current_platform();
            let ids = self
                .config
                .active_scheme_ids_by_platform
                .entry(platform)
                .or_default();

            if !ids.iter().any(|scheme_id| scheme_id == id) {
                ids.push(id.to_string());
            }
        } else {
            let platform = Self::current_platform();
            if let Some(ids) = self.config.active_scheme_ids_by_platform.get_mut(&platform) {
                ids.retain(|scheme_id| scheme_id != id);
            }
        }

        self.apply_active_schemes_for_current_platform()?;
        self.sync_enabled_flags();
        self.save_config().and_then(|_| self.get_all_schemes())
    }

    pub fn update_scheme_remote_config(
        &mut self,
        id: &str,
        remote_url: Option<String>,
        auto_sync_enabled: bool,
        sync_interval_minutes: Option<u64>,
    ) -> io::Result<Scheme> {
        let normalized_url = remote_url.and_then(|url| {
            let trimmed = url.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });

        if auto_sync_enabled && (normalized_url.is_none() || sync_interval_minutes.unwrap_or(0) == 0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "启用自动同步前请先填写远程 URL 和同步间隔",
            ));
        }

        if let Some(scheme) = self.config.schemes.iter_mut().find(|scheme| scheme.id == id) {
            scheme.remote_url = normalized_url;
            scheme.auto_sync_enabled = auto_sync_enabled;
            scheme.sync_interval_minutes = if auto_sync_enabled {
                sync_interval_minutes
            } else {
                sync_interval_minutes.filter(|interval| *interval > 0)
            };

            if scheme.remote_url.is_none() {
                scheme.auto_sync_enabled = false;
                scheme.sync_interval_minutes = None;
                scheme.last_sync_error = None;
                scheme.sync_status = "idle".to_string();
                scheme.last_sync_message = None;
                scheme.next_retry_at = None;
                scheme.consecutive_failures = 0;
            }

            scheme.updated_at = Utc::now();
            let updated = scheme.clone();
            self.save_config()?;
            Ok(updated)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "Scheme not found"))
        }
    }

    pub fn mark_remote_sync_started(&mut self, id: &str, trigger: &str) -> io::Result<Scheme> {
        if let Some(scheme) = self.config.schemes.iter_mut().find(|scheme| scheme.id == id) {
            if scheme.sync_status == "syncing" {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "当前分组正在同步中",
                ));
            }

            scheme.sync_status = "syncing".to_string();
            scheme.last_sync_message = Some(match trigger {
                "scheduled" => "后台定时同步中".to_string(),
                "retry" => "后台重试同步中".to_string(),
                _ => "手动同步中".to_string(),
            });
            scheme.updated_at = Utc::now();
            let updated = scheme.clone();
            self.save_config()?;
            Ok(updated)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "Scheme not found"))
        }
    }

    pub fn apply_remote_scheme_content_with_trigger(
        &mut self,
        id: &str,
        content: String,
        trigger: &str,
    ) -> io::Result<Scheme> {
        let is_enabled = self
            .active_ids_for_platform(&Self::current_platform())
            .iter()
            .any(|scheme_id| scheme_id == id);

        if let Some(scheme) = self.config.schemes.iter_mut().find(|scheme| scheme.id == id) {
            scheme.content = content;
            scheme.last_synced_at = Some(Utc::now());
            scheme.last_sync_error = None;
            scheme.sync_status = "success".to_string();
            scheme.last_sync_message = Some(if is_enabled {
                "远程同步成功，已自动应用到当前系统 Hosts".to_string()
            } else {
                "远程同步成功".to_string()
            });
            scheme.next_retry_at = None;
            scheme.consecutive_failures = 0;
            Self::push_sync_log(
                scheme,
                "success",
                trigger,
                scheme.last_sync_message.clone().unwrap_or_else(|| "远程同步成功".to_string()),
            );
            scheme.updated_at = Utc::now();
        } else {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Scheme not found"));
        }

        if is_enabled {
            self.apply_active_schemes_for_current_platform()?;
        }

        self.sync_enabled_flags();
        if let Some(scheme) = self.config.schemes.iter_mut().find(|scheme| scheme.id == id) {
            Self::flush_scheme_logs_at(&self.sync_log_path, scheme)?;
        }
        self.save_config()?;
        self.get_scheme(id)
    }

    pub fn mark_remote_sync_error_with_trigger(
        &mut self,
        id: &str,
        error_message: String,
        trigger: &str,
    ) -> io::Result<Scheme> {
        if let Some(scheme) = self.config.schemes.iter_mut().find(|scheme| scheme.id == id) {
            scheme.last_sync_error = Some(error_message);
            scheme.sync_status = "error".to_string();
            scheme.last_sync_message = scheme.last_sync_error.clone();
            scheme.consecutive_failures = scheme.consecutive_failures.saturating_add(1);
            scheme.next_retry_at = if scheme.auto_sync_enabled {
                Some(Utc::now() + Duration::minutes(Self::retry_delay_minutes(scheme.consecutive_failures)))
            } else {
                None
            };
            let message = scheme.last_sync_error.clone().unwrap_or_else(|| "未知同步错误".to_string());
            Self::push_sync_log(scheme, "error", trigger, message);
            scheme.updated_at = Utc::now();
            Self::flush_scheme_logs_at(&self.sync_log_path, scheme)?;
            self.save_config()?;
            self.get_scheme(id)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "Scheme not found"))
        }
    }

    pub fn export_schemes(&self, export_path: PathBuf) -> io::Result<()> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(export_path, content)
    }

    pub fn import_schemes(&mut self, import_path: PathBuf) -> io::Result<Vec<Scheme>> {
        let content = fs::read_to_string(import_path)?;
        let imported_config: SchemeConfig = serde_json::from_str(&content).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid import file: {}", error),
            )
        })?;

        for imported_scheme in imported_config.schemes {
            let mut scheme = imported_scheme;
            scheme.id = uuid::Uuid::new_v4().to_string();
            scheme.enabled = false;
            scheme.name = self.make_unique_scheme_name(&scheme.name);
            scheme.updated_at = Utc::now();
            self.config.schemes.push(scheme);
        }

        self.sync_enabled_flags();
        self.save_config().and_then(|_| self.get_all_schemes())
    }

    fn save_config(&self) -> io::Result<()> {
        let mut config_to_save = self.config.clone();
        for scheme in &mut config_to_save.schemes {
            scheme.sync_logs.clear();
        }

        let content = serde_json::to_string_pretty(&config_to_save)?;
        fs::write(&self.config_path, &content)
    }

    fn load_sync_logs_from(path: &PathBuf) -> io::Result<HashMap<String, Vec<SyncLogEntry>>> {
        if !path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse sync log file: {}", error),
            )
        })
    }

    fn save_sync_logs_to(path: &PathBuf, logs: &HashMap<String, Vec<SyncLogEntry>>) -> io::Result<()> {
        let content = serde_json::to_string_pretty(logs)?;
        fs::write(path, content)
    }

    fn current_platform() -> String {
        std::env::consts::OS.to_string()
    }

    fn active_ids_for_platform(&self, platform: &str) -> Vec<String> {
        self.config
            .active_scheme_ids_by_platform
            .get(platform)
            .cloned()
            .unwrap_or_default()
    }

    fn sync_enabled_flags(&mut self) {
        let active_ids = self.active_ids_for_platform(&Self::current_platform());
        for scheme in &mut self.config.schemes {
            scheme.enabled = active_ids.iter().any(|active_id| active_id == &scheme.id);
        }
    }

    fn migrate_legacy_platform_state(&mut self) {
        let platform = Self::current_platform();
        let already_migrated = self
            .config
            .version
            .split('.')
            .next()
            .and_then(|major| major.parse::<u64>().ok())
            .map(|major| major >= 2)
            .unwrap_or(false);

        if already_migrated {
            self.sync_enabled_flags();
            return;
        }

        let needs_migration = self
            .config
            .active_scheme_ids_by_platform
            .get(&platform)
            .is_none()
            && !self.config.active_scheme_ids.is_empty();

        if needs_migration {
            self.config
                .active_scheme_ids_by_platform
                .insert(platform, self.config.active_scheme_ids.clone());
            self.config.version = "2.0".to_string();
        }

        self.sync_enabled_flags();
    }

    fn apply_active_schemes_for_current_platform(&self) -> io::Result<()> {
        let active_ids = self.active_ids_for_platform(&Self::current_platform());
        if active_ids.is_empty() {
            return hosts::write_managed_hosts_file("");
        }

        let mut merged_contents = Vec::new();
        for active_id in active_ids {
            let scheme = self
                .config
                .schemes
                .iter()
                .find(|scheme| scheme.id == active_id)
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Scheme not found"))?;

            merged_contents.push(format!("# Group: {}\n{}", scheme.name, scheme.content.trim()));
        }

        let merged = merged_contents.join("\n\n");
        validate_hosts_content(&merged)
            .map_err(|message| io::Error::new(io::ErrorKind::InvalidData, message))?;
        hosts::write_managed_hosts_file(&merged)
    }

    fn make_unique_scheme_name(&self, base_name: &str) -> String {
        if !self.config.schemes.iter().any(|scheme| scheme.name == base_name) {
            return base_name.to_string();
        }

        let mut index = 1;
        loop {
            let candidate = format!("{} ({})", base_name, index);
            if !self
                .config
                .schemes
                .iter()
                .any(|scheme| scheme.name == candidate)
            {
                return candidate;
            }
            index += 1;
        }
    }

    fn validate_scheme_name(&self, name: &str, exclude_id: Option<&str>) -> io::Result<String> {
        let trimmed_name = name.trim();
        if trimmed_name.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "分组名称不能为空"));
        }

        let normalized = trimmed_name.to_lowercase();
        let duplicated = self.config.schemes.iter().any(|scheme| {
            if exclude_id == Some(scheme.id.as_str()) {
                return false;
            }

            scheme.name.trim().to_lowercase() == normalized
        });

        if duplicated {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("已存在同名分组「{}」", trimmed_name),
            ));
        }

        Ok(trimmed_name.to_string())
    }

    fn push_sync_log(scheme: &mut Scheme, status: &str, trigger: &str, message: String) {
        scheme.sync_logs.insert(
            0,
            SyncLogEntry {
                timestamp: Utc::now(),
                status: status.to_string(),
                trigger: trigger.to_string(),
                message,
            },
        );

        if scheme.sync_logs.len() > Self::MAX_SYNC_LOGS {
            scheme.sync_logs.truncate(Self::MAX_SYNC_LOGS);
        }
    }

    fn flush_scheme_logs_at(path: &PathBuf, scheme: &mut Scheme) -> io::Result<()> {
        if scheme.sync_logs.is_empty() {
            return Ok(());
        }

        let mut logs = Self::load_sync_logs_from(path)?;
        let entries = logs.entry(scheme.id.clone()).or_default();
        for log in scheme.sync_logs.drain(..) {
            entries.insert(0, log);
        }

        if entries.len() > Self::MAX_SYNC_LOGS {
            entries.truncate(Self::MAX_SYNC_LOGS);
        }

        Self::save_sync_logs_to(path, &logs)
    }

    fn migrate_embedded_logs_to_store(&mut self) {
        let mut changed = false;

        for index in 0..self.config.schemes.len() {
            if !self.config.schemes[index].sync_logs.is_empty() {
                let _ = Self::flush_scheme_logs_at(&self.sync_log_path, &mut self.config.schemes[index]);
                changed = true;
            }
        }

        if changed {
            let _ = self.save_config();
        }
    }

    fn reset_sync_runtime_state(&mut self) {
        for scheme in &mut self.config.schemes {
            if scheme.sync_status == "syncing" {
                scheme.sync_status = "idle".to_string();
                scheme.last_sync_message = Some("应用重新启动，已重置同步状态".to_string());
            }
        }

        let _ = self.save_config();
    }

    fn retry_delay_minutes(consecutive_failures: u32) -> i64 {
        match consecutive_failures {
            0 | 1 => 1,
            2 => 2,
            3 => 5,
            _ => 10,
        }
    }

    fn get_next_sync_wait_duration_from(&self, now: DateTime<Utc>) -> std::time::Duration {
        let minimum = std::time::Duration::from_secs(MIN_BACKGROUND_SYNC_WAIT_SECS);
        let maximum = std::time::Duration::from_secs(MAX_BACKGROUND_SYNC_WAIT_SECS);

        let next_due_at = self
            .config
            .schemes
            .iter()
            .filter_map(|scheme| self.next_due_at_for_scheme(scheme, now))
            .min();

        let Some(next_due_at) = next_due_at else {
            return maximum;
        };

        if next_due_at <= now {
            return minimum;
        }

        let until_due = (next_due_at - now)
            .to_std()
            .unwrap_or(minimum);

        until_due.clamp(minimum, maximum)
    }

    fn next_due_at_for_scheme(&self, scheme: &Scheme, _now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        let remote_url = scheme.remote_url.as_ref()?.trim();
        let interval = scheme.sync_interval_minutes?;
        if !scheme.auto_sync_enabled || remote_url.is_empty() || interval == 0 {
            return None;
        }

        if scheme.sync_status == "syncing" {
            return None;
        }

        if let Some(next_retry_at) = scheme.next_retry_at {
            return Some(next_retry_at);
        }

        let last_synced_at = scheme
            .last_synced_at
            .unwrap_or_else(|| DateTime::<Utc>::from(std::time::SystemTime::UNIX_EPOCH));

        Some(last_synced_at + Duration::minutes(interval as i64))
    }

    fn get_config_dir() -> io::Result<PathBuf> {
        match dirs::config_dir() {
            Some(dir) => Ok(dir.join("rust-switchhost")),
            None => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Failed to get config directory",
            )),
        }
    }
}

impl Default for SchemeManager {
    fn default() -> Self {
        Self::new().expect("Failed to create SchemeManager")
    }
}

#[cfg(test)]
mod tests {
    use super::{SchemeManager, MAX_BACKGROUND_SYNC_WAIT_SECS, MIN_BACKGROUND_SYNC_WAIT_SECS};
    use crate::schemes::{Scheme, SchemeConfig};
    use chrono::{Duration, Utc};
    use std::path::PathBuf;

    #[test]
    fn calculates_retry_delay_with_backoff() {
        assert_eq!(SchemeManager::retry_delay_minutes(0), 1);
        assert_eq!(SchemeManager::retry_delay_minutes(1), 1);
        assert_eq!(SchemeManager::retry_delay_minutes(2), 2);
        assert_eq!(SchemeManager::retry_delay_minutes(3), 5);
        assert_eq!(SchemeManager::retry_delay_minutes(8), 10);
    }

    #[test]
    fn next_sync_wait_duration_uses_nearest_due_scheme() {
        let now = Utc::now();
        let mut manager = SchemeManager {
            config: SchemeConfig::default(),
            config_path: PathBuf::new(),
            sync_log_path: PathBuf::new(),
        };

        let mut scheme = Scheme::new("remote".to_string(), "127.0.0.1 example.test".to_string());
        scheme.remote_url = Some("https://example.com/hosts".to_string());
        scheme.auto_sync_enabled = true;
        scheme.sync_interval_minutes = Some(15);
        scheme.last_synced_at = Some(now - Duration::minutes(14) - Duration::seconds(56));
        manager.config.schemes.push(scheme);

        let wait = manager.get_next_sync_wait_duration_from(now);
        assert_eq!(wait.as_secs(), MIN_BACKGROUND_SYNC_WAIT_SECS);
    }

    #[test]
    fn next_sync_wait_duration_caps_idle_wait() {
        let manager = SchemeManager {
            config: SchemeConfig::default(),
            config_path: PathBuf::new(),
            sync_log_path: PathBuf::new(),
        };

        let wait = manager.get_next_sync_wait_duration_from(Utc::now());
        assert_eq!(wait.as_secs(), MAX_BACKGROUND_SYNC_WAIT_SECS);
    }
}
