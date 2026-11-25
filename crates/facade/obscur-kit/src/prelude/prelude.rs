//! The stable prelude: `use obscur_kit::prelude::*;` brings the most-used types
//! from across the toolkit into scope (spec §2.9).

pub use crate::geometry::{Point, Rect, ScaleFactor, Size};
pub use crate::identity::{ComponentId, NodeId, RuntimeId, WindowId};
pub use crate::layout::model::LayoutStyle;
pub use crate::localization::catalog::{Localizer, MessageCatalog};
pub use crate::localization::locale::LanguageTag;
pub use crate::localization::message::MessageArgs;
pub use crate::scene::Color;
pub use crate::scene::scene::{Scene, SceneSnapshot};
pub use crate::style::ComputedStyle;
pub use crate::text::span::TextStyle;
pub use crate::tree::tree::RetainedTree;
pub use crate::view::component::Component;
pub use crate::view::view::View;
pub use crate::windowing::window::{Window, WindowProperties};
// interaction — §2.3's event model as a handler writes it: the event itself, the
// typed payload it carries, the phase a listener asks for, and the two enums a
// handler compares against (`e.trust() == EventTrust::Trusted`,
// `e.phase() == EventPhase::Capture`). Only `UiEvent` used to be here, which left
// every handler signature spelling out a five-segment path for the rest.
pub use crate::interaction::event::{EventPayload, EventPhase, EventTrust, ListenerPhase, UiEvent};
pub use crate::interaction::pointer::PointerEvent;
pub use crate::render::Renderer;
pub use crate::render::target::RenderTarget;
pub use crate::resources::hub::ResourceHub;
pub use crate::runtime::frame::{FrameContext, FramePipeline, FrameReason, FrameResult};
// runtime command handlers — in scope so a widened registration
// (`on_command(CMD, |e: &mut UiEvent| …)`) resolves without naming the marker trait.
pub use crate::runtime::IntoCommandHandler;
// live control of a running runtime, and the run-time `.okui` screen
pub use crate::control::AppControl;
pub use crate::screen::{OkuiScreen, ScreenData};
// headless host, and the §7.5 stable identifier a positional registration names
pub use crate::headless::HeadlessHost;
pub use crate::headless::input::SelectorTarget;
