use ipaws_core::channels::IpawsConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(flatten)]
    pub config: IpawsConfig,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self { config: IpawsConfig { use_test_endpoint: true, use_file_cert: true,
                                     confirm_before_send: true, ..Default::default() } }
    }
}

impl AppSettings {
    fn settings_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("IpawsAlert").join("settings.json"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::settings_path() else { return Self::default() };
        let Ok(data)   = std::fs::read_to_string(&path) else { return Self::default() };
        serde_json::from_str(&data).unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::settings_path().ok_or("Cannot determine config dir")?;
        if let Some(p) = path.parent() { std::fs::create_dir_all(p).map_err(|e| e.to_string())?; }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, json).map_err(|e| e.to_string())
    }
}
