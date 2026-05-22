use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    if std::env::var("CARGO_CFG_WINDOWS").is_ok() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let icon_path = Path::new(&manifest_dir).join("icon.ico");
        
        let ico_bytes = generate_ico_file();
        let mut file = File::create(&icon_path).unwrap();
        file.write_all(&ico_bytes).unwrap();

        let mut res = winres::WindowsResource::new();
        res.set_icon(icon_path.to_str().unwrap());
        res.compile().unwrap();
    }
}

fn generate_ico_file() -> Vec<u8> {
    let size = 32;
    let mut rgba = vec![0u8; size * size * 4];

    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;

            // Normalize coordinates to -1.0 to 1.0 range
            let cx = size as f32 / 2.0;
            let cy = size as f32 / 2.0;
            let dx = (x as f32 - cx) / cx; // Horizontal (-1 to 1)
            let dy = (y as f32 - cy) / cy; // Vertical (-1 to 1)

            let mut inside_boat = false;
            let mut color_t = 0.0; // For gradient mapping

            // 1. THE HULL (A trapezoid-like shape at the bottom)
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
            if dx > 0.06 && dx < 0.7 && dy > -0.7 && dy < 0.1 {
                let sail_width = 0.7 * (1.0 - (dy + 0.7) / 0.8);
                if dx - 0.06 < sail_width {
                    inside_boat = true;
                    color_t = 0.3;
                }
            }

            if inside_boat {
                rgba[idx] = (40.0 + color_t * 30.0) as u8; // R
                rgba[idx + 1] = (120.0 + color_t * 60.0) as u8; // G
                rgba[idx + 2] = (220.0 - color_t * 40.0) as u8; // B
                rgba[idx + 3] = 255; // A
            } else {
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 0;
            }
        }
    }

    // Now construct ICO bytes.
    let mut ico = Vec::new();

    // ICONHEADER (6 bytes)
    ico.extend_from_slice(&[0, 0]); // Reserved
    ico.extend_from_slice(&[1, 0]); // Type (1 for icon)
    ico.extend_from_slice(&[1, 0]); // Count (1 image)

    // ICONDIRENTRY (16 bytes)
    ico.push(32); // Width (32)
    ico.push(32); // Height (32)
    ico.push(0);  // Color count (0 for >=8bpp)
    ico.push(0);  // Reserved
    ico.extend_from_slice(&[1, 0]);  // Color planes (1)
    ico.extend_from_slice(&[32, 0]); // Bits per pixel (32)
    
    // Image size: 40 (BITMAPINFOHEADER) + 4096 (XOR mask) + 128 (AND mask) = 4264 bytes
    let image_size = 4264u32;
    ico.extend_from_slice(&image_size.to_le_bytes());
    
    // Image offset: 6 (ICONHEADER) + 16 (ICONDIRENTRY) = 22
    let image_offset = 22u32;
    ico.extend_from_slice(&image_offset.to_le_bytes());

    // Image Data:
    // BITMAPINFOHEADER (40 bytes)
    ico.extend_from_slice(&40u32.to_le_bytes()); // biSize
    ico.extend_from_slice(&32i32.to_le_bytes()); // biWidth
    ico.extend_from_slice(&64i32.to_le_bytes()); // biHeight (32 * 2 = 64)
    ico.extend_from_slice(&1u16.to_le_bytes());  // biPlanes
    ico.extend_from_slice(&32u16.to_le_bytes()); // biBitCount
    ico.extend_from_slice(&0u32.to_le_bytes());  // biCompression (0 = BI_RGB)
    ico.extend_from_slice(&4096u32.to_le_bytes()); // biSizeImage (32 * 32 * 4)
    ico.extend_from_slice(&0i32.to_le_bytes());  // biXPelsPerMeter
    ico.extend_from_slice(&0i32.to_le_bytes());  // biYPelsPerMeter
    ico.extend_from_slice(&0u32.to_le_bytes());  // biClrUsed
    ico.extend_from_slice(&0u32.to_le_bytes());  // biClrImportant

    // XOR mask: Pixel Data in BGRA format, bottom-to-top
    for y_inv in 0..size {
        let y = size - 1 - y_inv;
        for x in 0..size {
            let idx = (y * size + x) * 4;
            let r = rgba[idx];
            let g = rgba[idx + 1];
            let b = rgba[idx + 2];
            let a = rgba[idx + 3];
            ico.push(b);
            ico.push(g);
            ico.push(r);
            ico.push(a);
        }
    }

    // AND mask: 1 bit per pixel, 32 rows, each row padded to 32 bits (4 bytes)
    for y_inv in 0..size {
        let y = size - 1 - y_inv;
        let mut row_byte = 0u8;
        for x in 0..size {
            let idx = (y * size + x) * 4;
            let a = rgba[idx + 3];
            let bit = if a == 0 { 1 } else { 0 };
            
            let bit_pos = 7 - (x % 8);
            row_byte |= bit << bit_pos;

            if x % 8 == 7 {
                ico.push(row_byte);
                row_byte = 0;
            }
        }
    }

    ico
}
