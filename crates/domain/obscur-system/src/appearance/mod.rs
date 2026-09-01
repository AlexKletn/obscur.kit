#![doc = include_str!("mod.doc/_module.md")]
mod appearance_provider;
pub use appearance_provider::{AppearanceCallback, AppearanceProvider, NullAppearanceProvider};
mod appearance_watch;
pub use appearance_watch::AppearanceWatch;
mod color_scheme;
pub use color_scheme::ColorScheme;
mod contrast;
pub use contrast::Contrast;
mod system_appearance;
pub use system_appearance::SystemAppearance;
mod system_font;
pub use system_font::SystemFont;
mod system_palette;
pub use system_palette::SystemPalette;
