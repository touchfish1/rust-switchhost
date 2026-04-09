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
}

impl Default for SchemeManager {
    fn default() -> Self {
        Self::new().expect("Failed to create SchemeManager")
    }
}
