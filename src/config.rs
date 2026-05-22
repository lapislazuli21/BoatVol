use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub device_volumes: HashMap<String, f32>,
    #[serde(default = "default_true")]
    pub launch_at_startup: bool,
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
        let config = if config_path.exists() {
            let contents = fs::read_to_string(config_path).expect("Failed to read config file");
            let parsed: AppConfig =
                serde_json::from_str(&contents).expect("Failed to parse config file");
            parsed
        } else {
            AppConfig {
                device_volumes: HashMap::new(),
                launch_at_startup: true,
            }
        };

        // Automatically update startup entry to point to current exe path if enabled
        if config.launch_at_startup {
            let _ = update_startup(true);
            clean_old_startup_shortcuts();
        }

        config
    }

    pub fn save(&self) {
        let config_path = Self::get_config_path();
        let contents = serde_json::to_string_pretty(self).expect("Failed to serialize config");
        fs::write(config_path, contents).expect("Failed to write config file");
    }
}

pub fn clean_old_startup_shortcuts() {
    if let Some(mut path) = dirs::config_dir() {
        path.push("Microsoft");
        path.push("Windows");
        path.push("Start Menu");
        path.push("Programs");
        path.push("Startup");

        let file_names = ["boatvol.lnk", "BoatVol.lnk"];
        for name in &file_names {
            let shortcut_path = path.join(name);
            if shortcut_path.exists() {
                let _ = std::fs::remove_file(shortcut_path);
            }
        }
    }
}

pub fn update_startup(enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
    use windows::core::{HSTRING, PCWSTR};
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegSetValueExW, RegDeleteValueW, HKEY_CURRENT_USER, KEY_SET_VALUE, HKEY, REG_SZ
    };

    let subkey = HSTRING::from("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
    let name = HSTRING::from("BoatVol");

    unsafe {
        let mut hkey = HKEY::default();
        let status = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            None,
            KEY_SET_VALUE,
            &mut hkey,
        );
        if status.is_err() {
            return Err(format!("Failed to open registry key: {:?}", status).into());
        }

        if enabled {
            let current_exe = std::env::current_exe()?;
            let mut cmd = format!("\"{}\"", current_exe.to_string_lossy());
            if std::env::args().any(|arg| arg == "--save-on-change") {
                cmd.push_str(" --save-on-change");
            }
            
            // Encode command string as a null-terminated UTF-16 wide string and cast to &[u8]
            let mut cmd_wide: Vec<u16> = cmd.encode_utf16().collect();
            cmd_wide.push(0);
            let val_bytes = std::slice::from_raw_parts(
                cmd_wide.as_ptr() as *const u8,
                cmd_wide.len() * 2,
            );

            let set_status = RegSetValueExW(
                hkey,
                PCWSTR(name.as_ptr()),
                None,
                REG_SZ,
                Some(val_bytes),
            );
            let _ = RegCloseKey(hkey);
            if set_status.is_err() {
                return Err(format!("Failed to set registry value: {:?}", set_status).into());
            }
        } else {
            let _ = RegDeleteValueW(hkey, PCWSTR(name.as_ptr()));
            let _ = RegCloseKey(hkey);
        }
    }
    Ok(())
}


