mod manager;

pub use manager::SchemeManager;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scheme {
    pub id: String,
    pub name: String,
    pub content: String,
    #[serde(default)]
    pub remote_url: Option<String>,
    #[serde(default)]
    pub auto_sync_enabled: bool,
    #[serde(default)]
    pub sync_interval_minutes: Option<u64>,
    #[serde(default)]
    pub last_synced_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_sync_error: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Scheme {
    pub fn new(name: String, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            content,
            remote_url: None,
            auto_sync_enabled: false,
            sync_interval_minutes: None,
            last_synced_at: None,
            last_sync_error: None,
            enabled: false,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemeConfig {
    pub version: String,
    pub schemes: Vec<Scheme>,
    #[serde(default)]
    pub active_scheme_ids: Vec<String>,
    #[serde(default)]
    pub active_scheme_ids_by_platform: HashMap<String, Vec<String>>,
}

impl Default for SchemeConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            schemes: Vec::new(),
            active_scheme_ids: Vec::new(),
            active_scheme_ids_by_platform: HashMap::new(),
        }
    }
}
