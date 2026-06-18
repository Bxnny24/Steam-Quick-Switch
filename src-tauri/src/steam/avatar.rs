//! Loading and shaping Steam avatars from the local avatar cache.

use std::path::{Path, PathBuf};

use base64::Engine;

/// Path to the locally cached avatar PNG for a SteamID64, if it exists.
pub fn avatar_path(steam_path: &Path, steam_id64: &str) -> Option<PathBuf> {
    let path = steam_path
        .join("config")
        .join("avatarcache")
        .join(format!("{steam_id64}.png"));
    path.exists().then_some(path)
}

/// Read the cached avatar for a SteamID64 and encode it as a PNG data URL,
/// suitable for an `<img src>` in the frontend.
pub fn avatar_data_url(steam_path: &Path, steam_id64: &str) -> Option<String> {
    let path = avatar_path(steam_path, steam_id64)?;
    let bytes = std::fs::read(&path).ok()?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:image/png;base64,{encoded}"))
}

/// Decode an avatar PNG into a `size`x`size` round RGBA buffer suitable for a
/// tray icon. Returns `(rgba_bytes, size)`, or `None` if decoding fails.
pub fn round_icon_rgba(png_path: &Path, size: u32) -> Option<(Vec<u8>, u32)> {
    let img = image::open(png_path).ok()?.to_rgba8();
    let mut resized =
        image::imageops::resize(&img, size, size, image::imageops::FilterType::Lanczos3);
    apply_circle_mask(&mut resized, size);
    Some((resized.into_raw(), size))
}

/// Apply an anti-aliased circular alpha mask in place, so the square avatar
/// renders as a circle in the tray.
fn apply_circle_mask(img: &mut image::RgbaImage, size: u32) {
    let radius = size as f32 / 2.0;
    let center = radius - 0.5;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();
            let factor = if dist <= radius - 1.0 {
                1.0
            } else if dist >= radius {
                0.0
            } else {
                radius - dist
            };
            let px = img.get_pixel_mut(x, y);
            px[3] = (px[3] as f32 * factor) as u8;
        }
    }
}
