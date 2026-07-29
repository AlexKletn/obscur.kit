//! Layer-surface module: shell surfaces that are part of the desktop itself
//! rather than application windows (spec §6.5).
mod layer_anchor;
pub use layer_anchor::LayerAnchor;
mod layer_exclusive_zone;
pub use layer_exclusive_zone::LayerExclusiveZone;
mod layer_keyboard_interactivity;
pub use layer_keyboard_interactivity::LayerKeyboardInteractivity;
mod layer_kind;
pub use layer_kind::LayerKind;
mod layer_surface_config;
pub use layer_surface_config::{LayerSurfaceConfig, Margins};
