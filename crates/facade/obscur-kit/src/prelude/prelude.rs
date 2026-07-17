//! The stable prelude: `use obscur_kit::prelude::*;` brings the most-used types
//! from across the toolkit into scope (spec §2.9).

// geometry
pub use crate::geometry::{Point, Rect, ScaleFactor, Size};
// identity
pub use crate::identity::{ComponentId, NodeId, RuntimeId, WindowId};
// view / components
pub use crate::view::component::Component;
pub use crate::view::view::View;
// retained tree
pub use crate::tree::tree::RetainedTree;
// computed style + layout
pub use crate::layout::model::LayoutStyle;
pub use crate::style::ComputedStyle;
// scene / paint
pub use crate::scene::Color;
pub use crate::scene::scene::{Scene, SceneSnapshot};
// text
pub use crate::text::span::TextStyle;
// localization
pub use crate::localization::catalog::{Localizer, MessageCatalog};
pub use crate::localization::locale::LanguageTag;
pub use crate::localization::message::MessageArgs;
// windowing
pub use crate::windowing::window::{Window, WindowProperties};
// interaction
pub use crate::interaction::event::UiEvent;
pub use crate::interaction::pointer::PointerEvent;
// rendering
pub use crate::render::Renderer;
pub use crate::render::target::RenderTarget;
// resources
pub use crate::resources::hub::ResourceHub;
// runtime frame pipeline
pub use crate::runtime::frame::{FrameContext, FramePipeline, FrameReason, FrameResult};
// headless host
pub use crate::headless::HeadlessHost;
