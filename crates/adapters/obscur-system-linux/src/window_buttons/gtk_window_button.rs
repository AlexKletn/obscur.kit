//! GTK-backed snapshot provider for a GNOME Wayland session.

use std::{
    process::{Command, Stdio},
    sync::Arc,
};

use obscur_system::appearance::{ColorScheme, WindowButton};

use super::WindowButtonImages;

/// Capture actual GTK title button artwork without presenting a second window.
///
/// Requires GTK 4 and PyGObject in the current desktop session. Applications
/// can use their existing controls when the provider returns `None`.
#[must_use]
pub fn gtk_window_button_images(
    button: WindowButton,
    maximized: bool,
    focused: bool,
    scheme: ColorScheme,
    scale: u16,
) -> Option<WindowButtonImages> {
    if !std::env::var("XDG_CURRENT_DESKTOP").is_ok_and(|desktop| {
        desktop
            .split(':')
            .any(|name| name.trim().eq_ignore_ascii_case("GNOME"))
    }) {
        return None;
    }
    let role = match button {
        WindowButton::Close => "close",
        WindowButton::Maximize if maximized => "restore",
        WindowButton::Maximize => "maximize",
        WindowButton::Minimize => "minimize",
        _ => return None,
    };
    let scheme = match scheme {
        ColorScheme::Dark => "dark",
        ColorScheme::Light => "light",
    };
    let output = Command::new("python3")
        .args([
            "-c",
            include_str!("gtk_window_button.py"),
            role,
            scheme,
            if focused { "active" } else { "inactive" },
            &scale.clamp(1, 4).to_string(),
        ])
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    decode(&output.stdout)
}

fn decode(mut bytes: &[u8]) -> Option<WindowButtonImages> {
    let mut width = 0;
    let mut height = 0;
    let mut images = Vec::new();
    for _ in 0..4 {
        let (header, rest) = bytes.split_at_checked(12)?;
        width = u32::from_le_bytes(header[..4].try_into().ok()?);
        height = u32::from_le_bytes(header[4..8].try_into().ok()?);
        let len = u32::from_le_bytes(header[8..].try_into().ok()?) as usize;
        if !(1..=128).contains(&width) || !(1..=128).contains(&height) {
            return None;
        }
        let (png, rest) = rest.split_at_checked(len)?;
        if !png.starts_with(b"\x89PNG\r\n\x1a\n") {
            return None;
        }
        images.push(Arc::from(png));
        bytes = rest;
    }
    if !bytes.is_empty() {
        return None;
    }
    Some(WindowButtonImages {
        width,
        height,
        images: images.try_into().ok()?,
    })
}

#[cfg(test)]
#[path = "gtk_window_button.tests.rs"]
mod tests;
