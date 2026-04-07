use super::{Scheme, SchemeConfig};
use crate::hosts;
use std::fs;
use std::io;
use std::path::PathBuf;
use chrono::Utc;

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
                eprintln!("Warning: Failed to create config directory: {}, using in-memory config", e);
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
                    eprintln!("Warning: Failed to read config file: {}, using default config", e);
                    SchemeConfig::default()
                }
            }
        } else {
            SchemeConfig::default()
        };
        
        Ok(Self {
            config,
            config_path,
        })
    }
    
    pub fn get_all_schemes(&self) -> io::Result<Vec<Scheme>> {
        Ok(self.config.schemes.clone())
    }
    
    pub fn create_scheme(&mut self, name: String, content: String) -> io::Result<Scheme> {
        let scheme = Scheme::new(name, content);
        self.config.schemes.push(scheme.clone());
        self.save_config()?;
        Ok(scheme)
    }
    
    pub fn update_scheme(
        &mut self,
        id: &str,
        name: String,
        content: String,
    ) -> io::Result<Scheme> {
        if let Some(scheme) = self.config.schemes.iter_mut().find(|s| s.id == id) {
            scheme.name = name;
            scheme.content = content;
            scheme.updated_at = Utc::now();
            let updated = scheme.clone();
            self.save_config()?;
            Ok(updated)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Scheme not found",
            ))
        }
    }
    
    pub fn delete_scheme(&mut self, id: &str) -> io::Result<()> {
        self.config.schemes.retain(|s| s.id != id);
        self.config.active_scheme_ids.retain(|s| s != id);
        self.save_config()
    }
    
    pub fn switch_scheme(&mut self, id: &str) -> io::Result<()> {
        let scheme = self
            .config
            .schemes
            .iter()
            .find(|s| s.id == id)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Scheme not found"))?;
        
        hosts::write_hosts_file(&scheme.content)?;
        
        self.config.active_scheme_ids = vec![id.to_string()];
        self.save_config()
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
}

impl Default for SchemeManager {
    fn default() -> Self {
        Self::new().expect("Failed to create SchemeManager")
    }
}
