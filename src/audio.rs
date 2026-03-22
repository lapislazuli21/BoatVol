use std::sync::{Arc, Mutex};

use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Media::Audio::Endpoints::{
    IAudioEndpointVolume, IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl,
};
use windows::Win32::Media::Audio::{
    AUDIO_VOLUME_NOTIFICATION_DATA, DEVICE_STATE, EDataFlow, ERole, IMMDeviceEnumerator,
    IMMNotificationClient, IMMNotificationClient_Impl, MMDeviceEnumerator, eConsole, eRender,
};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
};
use windows::core::{PCWSTR, Result, implement};

use crate::config::AppConfig;

pub fn init_com() -> Result<()> {
    unsafe { CoInitializeEx(Some(std::ptr::null()), COINIT_MULTITHREADED).ok() }
}

pub fn get_endpoint_volume() -> Result<IAudioEndpointVolume> {
    unsafe {
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
        let volume = device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, Some(std::ptr::null()))?;
        Ok(volume)
    }
}

pub fn get_enumerator() -> Result<IMMDeviceEnumerator> {
    unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
}

pub fn get_volume(endpoint_vol: &IAudioEndpointVolume) -> Result<f32> {
    unsafe {
        let level = endpoint_vol.GetMasterVolumeLevelScalar()?;
        Ok(level)
    }
}

pub fn set_volume(endpoint_vol: &IAudioEndpointVolume, level: f32) -> Result<()> {
    unsafe {
        endpoint_vol.SetMasterVolumeLevelScalar(level, std::ptr::null())?;
        Ok(())
    }
}

#[implement(IAudioEndpointVolumeCallback)]
pub struct VolumeCallback {
    pub config: Arc<Mutex<AppConfig>>,
    pub device_id: String,
}

impl IAudioEndpointVolumeCallback_Impl for VolumeCallback_Impl {
    fn OnNotify(&self, data: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> windows::core::Result<()> {
        let data = unsafe { &*data };
        let new_volume = data.fMasterVolume;
        let mut config = self.config.lock().unwrap();
        config
            .device_volumes
            .insert(self.device_id.clone(), new_volume);
        config.save();
        println!(
            "Volume changed to {:.2} for device {}",
            new_volume, self.device_id
        );
        Ok(())
    }
}

pub fn register_device_change_callback(
    enumerator: &IMMDeviceEnumerator,
    callback: &IMMNotificationClient,
) -> Result<()> {
    unsafe {
        enumerator.RegisterEndpointNotificationCallback(callback)?;
    }
    Ok(())
}

#[implement(IMMNotificationClient)]
pub struct DeviceChangeCallback {
    pub config: Arc<Mutex<AppConfig>>,
    pub save_on_change: bool,
}

impl IMMNotificationClient_Impl for DeviceChangeCallback_Impl {
    fn OnDeviceStateChanged(&self, _: &PCWSTR, _: DEVICE_STATE) -> windows::core::Result<()> {
        Ok(())
    }
    fn OnDeviceAdded(&self, _: &PCWSTR) -> windows::core::Result<()> {
        Ok(())
    }
    fn OnDeviceRemoved(&self, _: &PCWSTR) -> windows::core::Result<()> {
        Ok(())
    }
    fn OnPropertyValueChanged(&self, _: &PCWSTR, _: &PROPERTYKEY) -> windows::core::Result<()> {
        Ok(())
    }

    fn OnDefaultDeviceChanged(
        &self,
        flow: EDataFlow,
        role: ERole,
        pwstrdefaultdeviceid: &PCWSTR,
    ) -> windows::core::Result<()> {
        if flow != eRender || role != eConsole {
            return Ok(());
        }
        let _ = init_com();

        let device_id = unsafe { pwstrdefaultdeviceid.to_string().unwrap_or_default() };
        println!("Default audio device changed to: {}", device_id);

        let config = self.config.clone();
        let save_on_change = self.save_on_change;
        std::thread::spawn(move || {
            let _ = init_com();

            let target_vol = {
                let mut config = config.lock().unwrap();
                if let Some(&saved_vol) = config.device_volumes.get(&device_id) {
                    saved_vol
                } else {
                    // New device — save a sensible default
                    let default_vol: f32 = 0.4;
                    config.device_volumes.insert(device_id.clone(), default_vol);
                    config.save();
                    println!("New device {}, will set to {}", device_id, default_vol);
                    default_vol
                }
            };

            for attempt in 1..=5 {
                std::thread::sleep(std::time::Duration::from_secs(2));

                let fresh_endpoint = match get_endpoint_volume() {
                    Ok(ep) => ep,
                    Err(e) => {
                        println!("Attempt {}: failed to get endpoint: {}", attempt, e);
                        continue;
                    }
                };

                let current = get_volume(&fresh_endpoint).unwrap_or(1.0);
                if (current - target_vol).abs() < 0.01 {
                    println!(
                        "Attempt {}: volume already at {:.2}, done!",
                        attempt, current
                    );
                    break;
                }

                set_volume(&fresh_endpoint, target_vol).unwrap();
                let actual = get_volume(&fresh_endpoint).unwrap_or(-1.0);
                println!(
                    "Attempt {}: set {} -> {:.2}, actual: {:.2}",
                    attempt, device_id, target_vol, actual
                );
            }

            if save_on_change {
                if let Ok(ep) = get_endpoint_volume() {
                    let cb = VolumeCallback {
                        config: config.clone(),
                        device_id: device_id.clone(),
                    };
                    let cb: IAudioEndpointVolumeCallback = cb.into();
                    unsafe {
                        let _ = ep.RegisterControlChangeNotify(&cb);
                    }
                    println!("Volume callback registered for {}", device_id);
                    // Park this thread forever to keep the callback alive
                    std::thread::park();
                }
            }
        });

        Ok(())
    }
}
