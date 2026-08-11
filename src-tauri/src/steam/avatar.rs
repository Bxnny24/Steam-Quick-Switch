//! Loading and shaping Steam avatars from the local avatar cache.

use std::path::{Path, PathBuf};

/// Path to the locally cached avatar PNG for a SteamID64, if it exists.
pub fn avatar_path(steam_path: &Path, steam_id64: &str) -> Option<PathBuf> {
    let path = steam_path
        .join("config")
        .join("avatarcache")
        .join(format!("{steam_id64}.png"));
    path.exists().then_some(path)
}

/// Path to Steam's own "no avatar" placeholder, which ships with the client.
/// Accounts without a profile picture then get Steam's familiar look instead of
/// a blank slot. Steam's grey silhouette default lives only on their CDN, so
/// this local `?` placeholder is the closest offline equivalent.
pub fn blank_avatar_path(steam_path: &Path) -> Option<PathBuf> {
    let path = steam_path.join("graphics").join("avatar_184blank.tga");
    path.exists().then_some(path)
}

/// Decode an avatar PNG into a `size`x`size` rounded-square RGBA buffer suitable
/// for a tray icon. Returns `(rgba_bytes, size)`, or `None` if decoding fails.
pub fn round_icon_rgba(png_path: &Path, size: u32) -> Option<(Vec<u8>, u32)> {
    let img = image::open(png_path).ok()?.to_rgba8();
    let mut resized =
        image::imageops::resize(&img, size, size, image::imageops::FilterType::Lanczos3);
    apply_rounded_mask(&mut resized, size);
    Some((resized.into_raw(), size))
}

/// Apply an anti-aliased rounded-rectangle (rounded square) alpha mask in place,
/// so the avatar renders with softly rounded corners instead of as a circle.
fn apply_rounded_mask(img: &mut image::RgbaImage, size: u32) {
    let half = size as f32 / 2.0;
    let center = half - 0.5;
    // Corner radius as a fraction of the icon size.
    let radius = (size as f32 * 0.28).max(2.0);
    for y in 0..size {
        for x in 0..size {
            // Signed distance to a rounded rectangle that fills the icon.
            let qx = (x as f32 - center).abs() - half + radius;
            let qy = (y as f32 - center).abs() - half + radius;
            let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
            let inside = qx.max(qy).min(0.0);
            let sdf = outside + inside - radius;
            // 1px anti-aliased edge: inside -> opaque, outside -> transparent.
            let factor = (0.5 - sdf).clamp(0.0, 1.0);
            let px = img.get_pixel_mut(x, y);
            px[3] = (px[3] as f32 * factor) as u8;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smoke test against the real machine. Ignored by default (needs Steam).
    /// Verifies the bundled TGA placeholder is found and decodes at icon sizes.
    /// Run with: cargo test -- --ignored --nocapture blank_avatar_decodes
    #[test]
    #[ignore]
    fn blank_avatar_decodes() {
        let steam = crate::steam::registry::steam_path().expect("Steam not found");
        let path = blank_avatar_path(&steam).expect("avatar_184blank.tga missing");
        println!("placeholder: {}", path.display());
        for size in [18u32, 32] {
            let (rgba, decoded) = round_icon_rgba(&path, size).expect("TGA decode failed");
            assert_eq!(decoded, size);
            assert_eq!(rgba.len(), (size * size * 4) as usize);
        }
    }
}
