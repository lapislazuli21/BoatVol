mod audio;
mod config;

use std::sync::{Arc, Mutex};
use windows::Win32::Media::Audio::IMMNotificationClient;

fn main() -> windows::core::Result<()> {
    let save_on_change = std::env::args().any(|arg| arg == "--save-on-change");

    audio::init_com()?;
    let config = Arc::new(Mutex::new(config::AppConfig::load()));

    // Register for default device changes
    let enumerator = audio::get_enumerator()?;
    let device_change_cb = audio::DeviceChangeCallback {
        config: config.clone(),
        save_on_change,
    };
    let device_change_cb: IMMNotificationClient = device_change_cb.into();
    audio::register_device_change_callback(&enumerator, &device_change_cb)?;

    println!(
        "BoatVol running (save-on-change: {}). Press Ctrl+C to stop.",
        save_on_change
    );
    std::thread::park();
    Ok(())
}
