//! Live control of a running runtime from application code (spec §2.6, §2.3, §4.12).
//!
//! An application configures itself *before* the loop starts, but three of the
//! things the spec asks for can only happen *while* it runs:
//!
//! - registering a handler at a node (§2.6's links 1–3), which needs a `NodeId`,
//!   which needs a committed frame;
//! - registering a §2.3 capture/bubble listener, for the same reason;
//! - replacing the authored stylesheets (§4.12's explicit invalidation).
//!
//! [`AppControl`] is the handle that makes those reachable without handing
//! application code a `&mut UiRuntime` it cannot legally hold: it records what to do
//! and the host applies it at one point in the frame — after a commit, before the
//! next build. Read [`AppControl`]'s own documentation for that contract; it is the
//! part that decides whether an application is correct.
//!
//! Nothing here needs a display: `App` composes it for a
//! standalone window, and a guest host embedding a
//! [`UiRuntime`](crate::runtime::UiRuntime) can use the same handle in its own loop.
//!
//! That asymmetry is why `App` is named in these docs as plain code rather than as
//! a link: this module compiles in every feature set and `App` exists only on
//! `window-winit` + `render-vello`, so an intra-doc link to it would be a broken
//! link in the default build — which is the doc-build warning this module shipped
//! with.
mod app_control;
pub use app_control::AppControl;
mod command_bindings;
pub(crate) use command_bindings::CommandBindings;
mod control_request;
pub(crate) use control_request::ControlRequest;
