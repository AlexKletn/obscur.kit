#![doc = include_str!("prelude.doc/_module.md")]

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
// interaction — §2.3's event model as a handler writes it: the event itself, the
// typed payload it carries, the phase a listener asks for, and the two enums a
// handler compares against (`e.trust() == EventTrust::Trusted`,
// `e.phase() == EventPhase::Capture`). Only `UiEvent` used to be here, which left
// every handler signature spelling out a five-segment path for the rest.
pub use crate::interaction::event::{EventPayload, EventPhase, EventTrust, ListenerPhase, UiEvent};
pub use crate::interaction::pointer::PointerEvent;
// rendering
pub use crate::render::Renderer;
pub use crate::render::target::RenderTarget;
// resources
pub use crate::resources::hub::ResourceHub;
// runtime frame pipeline
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
