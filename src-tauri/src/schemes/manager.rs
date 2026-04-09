use super::{Scheme, SchemeConfig};
use crate::hosts;
use chrono::Utc;
use std::fs;
use std::io;
use std::path::PathBuf;

pub struct SchemeManager {
    config: SchemeConfig,
    config_path: PathBuf,
}

impl SchemeManager {
    pub fn new() -> io::Result<Self> {
        let config_dir = match dirs::config_dir() {
            Some(dir) => dir.join("rust-switchhost"),
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Failed to get config directory",
                ))
            }
        };

        if !config_dir.exists() {
            if let Err(e) = fs::create_dir_all(&config_dir) {
                eprintln!(
                    "Warning: Failed to create config directory: {}, using in-memory config",
                    e
                );
                return Ok(Self {
                    config: SchemeConfig::default(),
                    config_path: config_dir.join("schemes.json"),
                });
            }
        }

        let config_path = config_dir.join("schemes.json");

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
        };
        manager.migrate_legacy_platform_state();
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

    pub fn create_scheme(&mut self, name: String, content: String) -> io::Result<Scheme> {
        let scheme = Scheme::new(name, content);
        self.config.schemes.push(scheme.clone());
        self.save_config()?;
        Ok(scheme)
    }

    pub fn update_scheme(&mut self, id: &str, name: String, content: String) -> io::Result<Scheme> {
        if let Some(scheme) = self.config.schemes.iter_mut().find(|s| s.id == id) {
            scheme.name = name;
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
        self.config.active_scheme_ids.retain(|s| s != id);
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
            .insert(platform, active_ids.clone());
        self.config.active_scheme_ids = active_ids;
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

            self.config.active_scheme_ids = ids.clone();
        } else {
            let platform = Self::current_platform();
            if let Some(ids) = self.config.active_scheme_ids_by_platform.get_mut(&platform) {
                ids.retain(|scheme_id| scheme_id != id);
                self.config.active_scheme_ids = ids.clone();
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
            }

            scheme.updated_at = Utc::now();
            let updated = scheme.clone();
            self.save_config()?;
            Ok(updated)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "Scheme not found"))
        }
    }

    pub fn apply_remote_scheme_content(&mut self, id: &str, content: String) -> io::Result<Scheme> {
        let is_enabled = self
            .active_ids_for_platform(&Self::current_platform())
            .iter()
            .any(|scheme_id| scheme_id == id);

        if let Some(scheme) = self.config.schemes.iter_mut().find(|scheme| scheme.id == id) {
            scheme.content = content;
            scheme.last_synced_at = Some(Utc::now());
            scheme.last_sync_error = None;
            scheme.updated_at = Utc::now();
        } else {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Scheme not found"));
        }

        if is_enabled {
            self.apply_active_schemes_for_current_platform()?;
        }

        self.sync_enabled_flags();
        self.save_config()?;
        self.get_scheme(id)
    }

    pub fn mark_remote_sync_error(&mut self, id: &str, error_message: String) -> io::Result<Scheme> {
        if let Some(scheme) = self.config.schemes.iter_mut().find(|scheme| scheme.id == id) {
            scheme.last_sync_error = Some(error_message);
            scheme.updated_at = Utc::now();
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
        let content = serde_json::to_string_pretty(&self.config)?;
        match fs::write(&self.config_path, &content) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Warning: Failed to save config: {}", e);
                Ok(())
            }
        }
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
        }

        self.sync_enabled_flags();
    }

    fn apply_active_schemes_for_current_platform(&self) -> io::Result<()> {
        let active_ids = self.active_ids_for_platform(&Self::current_platform());
        if active_ids.is_empty() {
            return hosts::write_hosts_file("");
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
        hosts::write_hosts_file(&merged)
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
}

impl Default for SchemeManager {
    fn default() -> Self {
        Self::new().expect("Failed to create SchemeManager")
    }
}
