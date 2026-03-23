use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIconBuilder};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, MSG, PostQuitMessage, TranslateMessage,
};

/// Generate a simple 32×32 RGBA icon (a blue-ish speaker/boat icon).
fn create_icon() -> Icon {
    let size = 32u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;

            // Normalize coordinates to -1.0 to 1.0 range
            let cx = size as f32 / 2.0;
            let cy = size as f32 / 2.0;
            let dx = (x as f32 - cx) / cx; // Horizontal (-1 to 1)
            let dy = (y as f32 - cy) / cy; // Vertical (-1 to 1)

            let mut inside_boat = false;
            let mut color_t = 0.0; // For gradient mapping

            // 1. THE HULL (A trapezoid-like shape at the bottom)
            // dy between 0.2 and 0.7
            if dy > 0.2 && dy < 0.7 {
                let width_at_y = 0.8 - (dy - 0.2) * 0.5; // Tapers toward the bottom
                if dx.abs() < width_at_y {
                    inside_boat = true;
                    color_t = (dy + 1.0) / 2.0; // Vertical gradient
                }
            }

            // 2. THE MAST (A thin rectangle)
            if dx.abs() < 0.06 && dy > -0.8 && dy <= 0.2 {
                inside_boat = true;
                color_t = 0.5;
            }

            // 3. THE SAIL (A triangle to the right of the mast)
            // dy between -0.7 and 0.1
            if dx > 0.06 && dx < 0.7 && dy > -0.7 && dy < 0.1 {
                // Triangle math: dx should be less than a value that shrinks as dy goes up
                let sail_width = 0.7 * (1.0 - (dy + 0.7) / 0.8);
                if dx - 0.06 < sail_width {
                    inside_boat = true;
                    color_t = 0.3; // Lighter/different part of gradient
                }
            }

            if inside_boat {
                // Applying your Blue-to-Teal gradient logic
                rgba[idx] = (40.0 + color_t * 30.0) as u8; // R
                rgba[idx + 1] = (120.0 + color_t * 60.0) as u8; // G
                rgba[idx + 2] = (220.0 - color_t * 40.0) as u8; // B
                rgba[idx + 3] = 255; // A
            } else {
                // Transparent background
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 0;
            }
        }
    }

    Icon::from_rgba(rgba, size, size).expect("Failed to create tray icon")
}

/// Runs the system tray with a Win32 message loop. Blocks until the user clicks "Quit".
pub fn run() {
    let quit_item = MenuItem::new("Quit", true, None);
    let quit_id = quit_item.id().clone();

    let menu = Menu::new();
    menu.append(&quit_item).expect("Failed to add menu item");

    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("BoatVol")
        .with_icon(create_icon())
        .build()
        .expect("Failed to create tray icon");

    // Win32 message loop — keeps the process alive and pumps tray events
    unsafe {
        let mut msg = MSG::default();
        loop {
            let ret = GetMessageW(&mut msg, None, 0, 0);
            if ret.0 <= 0 {
                break;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);

            // Check for tray menu events
            if let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id() == &quit_id {
                    PostQuitMessage(0);
                }
            }
        }
    }
}
