//! Resolve Linux desktop icon-theme files.

use std::{
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
};

use obscur_system::appearance::{SystemAppearance, WindowButton, WindowDecoration};

/// Resolve a named SVG in the given icon theme at the requested logical size and scale.
///
/// The lookup checks the selected theme, its declared parent themes, and the
/// standard fallback locations.
/// Missing icons and invalid lookup parameters return `None`.
#[must_use]
pub fn theme_icon_file(theme: &str, name: &str, size: u16, scale: u16) -> Option<PathBuf> {
    if !is_name(theme) || !is_name(name) || size == 0 || scale == 0 {
        return None;
    }
    freedesktop_icons::lookup(name)
        .with_theme(theme)
        .with_size(size)
        .with_scale(scale)
        .force_svg()
        .find()
        .filter(|path| path.extension().is_some_and(|extension| extension == "svg"))
}

/// Resolve the themed SVG for a window button from an appearance snapshot.
///
/// The maximize button selects the restore icon when `maximized` is true.
/// If the provider omits the icon name, the standard symbolic name is used.
/// Callers can supply their own fallback when the theme has no matching SVG.
#[must_use]
pub fn window_button_icon_file(
    appearance: &SystemAppearance,
    button: WindowButton,
    maximized: bool,
    size: u16,
    scale: u16,
) -> Option<PathBuf> {
    theme_icon_file(
        appearance.icon_theme.as_deref()?,
        window_button_icon_name(appearance.window_decoration.as_ref(), button, maximized)?,
        size,
        scale,
    )
}

/// Resolve and read a themed SVG on behalf of a resource loader.
///
/// The standard GNOME symbolic foreground is translated to `currentColor`
/// so the renderer can apply the widget's foreground on light and dark surfaces.
#[must_use]
pub fn theme_icon_svg(theme: &str, name: &str, size: u16, scale: u16) -> Option<Vec<u8>> {
    const MAX_BYTES: u64 = 1024 * 1024;
    let path = theme_icon_file(theme, name, size, scale)?;
    let file = File::open(path).ok()?;
    if !file.metadata().ok()?.is_file() {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes).ok()?;
    if bytes.len() as u64 > MAX_BYTES {
        return None;
    }
    if name.ends_with("-symbolic") {
        let svg = String::from_utf8(bytes).ok()?;
        return Some(svg.replace("#2e3436", "currentColor").into_bytes());
    }
    Some(bytes)
}

/// Return the desktop's name or the standard symbolic name for a window action.
#[must_use]
pub fn window_button_icon_name(
    decoration: Option<&WindowDecoration>,
    button: WindowButton,
    maximized: bool,
) -> Option<&str> {
    let icons = decoration.map(|decoration| &decoration.button_icons);
    let (name, fallback) = match button {
        WindowButton::Minimize => (
            icons.and_then(|icons| icons.minimize.as_deref()),
            "window-minimize-symbolic",
        ),
        WindowButton::Maximize if maximized => (
            icons.and_then(|icons| icons.restore.as_deref()),
            "window-restore-symbolic",
        ),
        WindowButton::Maximize => (
            icons.and_then(|icons| icons.maximize.as_deref()),
            "window-maximize-symbolic",
        ),
        WindowButton::Close => (
            icons.and_then(|icons| icons.close.as_deref()),
            "window-close-symbolic",
        ),
        _ => return None,
    };
    Some(name.unwrap_or(fallback))
}

fn is_name(value: &str) -> bool {
    let mut components = Path::new(value).components();
    matches!(components.next(), Some(Component::Normal(name)) if name == value)
        && components.next().is_none()
        && !value.contains('\0')
}

#[cfg(test)]
#[path = "icons.tests.rs"]
mod tests;
