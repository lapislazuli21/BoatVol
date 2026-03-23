use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub device_volumes: HashMap<String, f32>,
}

impl AppConfig {
    fn get_config_path() -> PathBuf {
        let mut path = dirs::config_dir().expect("Could not find AppData directory");
        path.push("BoatVol");

        if !path.exists() {
            std::fs::create_dir_all(&path).expect("Failed to create AppData/BoatVol directory");
        }

        path.push("config.json");
        path
    }

    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        if config_path.exists() {
            let contents = fs::read_to_string(config_path).expect("Failed to read config file");
            let parsed: AppConfig =
                serde_json::from_str(&contents).expect("Failed to parse config file");
            parsed
        } else {
            AppConfig {
                device_volumes: HashMap::new(),
            }
        }
    }

    pub fn save(&self) {
        let config_path = Self::get_config_path();
        let contents = serde_json::to_string_pretty(self).expect("Failed to serialize config");
        fs::write(config_path, contents).expect("Failed to write config file");
    }
}
