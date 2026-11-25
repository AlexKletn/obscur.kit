mod border_radius;
mod window_radius;

use window_radius::WindowRadius;

use std::path::PathBuf;

pub(super) fn window_radius(theme: &str) -> Option<f32> {
    let configured = std::env::var("GTK_THEME")
        .ok()
        .filter(|value| !value.is_empty());
    let theme = configured.as_deref().unwrap_or(theme);
    let name = theme.split(':').next()?;
    if !matches!(name, "Adwaita" | "Adwaita-dark") {
        return None;
    }
    let mut style = WindowRadius::default();
    style.cascade(super::adwaita_stylesheet::load()?);
    if let Some(css) = user_stylesheet() {
        style.cascade(&css);
    }
    style.resolve()
}

fn user_stylesheet() -> Option<String> {
    let root = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    std::fs::read_to_string(root.join("gtk-4.0/gtk.css")).ok()
}

#[cfg(test)]
#[path = "adwaita_theme.tests.rs"]
mod tests;
