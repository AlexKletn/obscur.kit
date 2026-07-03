//! Pure obscur-scene → Vello/peniko/kurbo converters (spec §2.7).
//!
//! All Scene/Paint → Vello conversions are local to this crate (spec §2.7) and
//! side-effect free. Each converter translates one part of the backend-agnostic
//! scene model into its Vello/peniko/kurbo equivalent while preserving geometry
//! and paint order (spec §5.2 determinism boundary); only rasterization is left
//! to Vello.
//!
//! - [`ColorConverter`] — [`Color`](obscur_scene::color::Color) → peniko color.
//! - [`PathConverter`] — [`Path`](obscur_scene::path::Path) → kurbo `BezPath`.
//! - [`BrushConverter`] — [`Brush`](obscur_scene::paint::Brush)/`Stroke` →
//!   peniko `Brush`/kurbo `Stroke`.
//! - [`ClipConverter`] — [`Clip`](obscur_scene::clip::Clip) → kurbo clip shape.
//! - [`SceneConverter`] — [`DisplayList`](obscur_scene::command::DisplayList) →
//!   `vello::Scene`, driving the above in paint order.
mod brush_converter;
pub use brush_converter::BrushConverter;
mod clip_converter;
pub use clip_converter::ClipConverter;
mod color_converter;
pub use color_converter::ColorConverter;
mod path_converter;
pub use path_converter::PathConverter;
mod scene_converter;
pub use scene_converter::SceneConverter;
