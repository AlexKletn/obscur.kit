//! Theme assets: bundled fonts, icons, images and stylesheets (spec §2.6).

mod theme_asset;
pub use theme_asset::ThemeAsset;
mod asset_kind;
pub use asset_kind::AssetKind;
mod asset_source;
pub use asset_source::AssetSource;
