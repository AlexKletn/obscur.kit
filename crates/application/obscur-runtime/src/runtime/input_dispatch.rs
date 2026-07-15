//! [`InputDispatch`]: what dispatching one queued [`InputEvent`] produced.

use obscur_identity::NodeId;

use super::{KeyDispatch, PointerDispatch};

/// The result of dispatching one queued
/// [`InputEvent`](super::InputEvent), in the order the events were queued
/// (spec §3.4 phase 3).
///
/// Each variant carries exactly what the matching immediate entry point returns —
/// [`handle_pointer`](super::UiRuntime::handle_pointer) and friends — because the
/// queue dispatches through those same functions. Queuing an event and handling it
/// immediately differ in *when*, never in *what*.
#[derive(Debug, Clone, PartialEq)]
pub enum InputDispatch {
    /// What [`handle_pointer`](super::UiRuntime::handle_pointer) returned.
    Pointer(PointerDispatch),
    /// What [`handle_key`](super::UiRuntime::handle_key) returned.
    Key(KeyDispatch),
    /// Whether [`handle_ime`](super::UiRuntime::handle_ime) consumed the
    /// composition.
    Composition(bool),
    /// The node [`handle_scroll`](super::UiRuntime::handle_scroll) scrolled, if
    /// any.
    Scroll(Option<NodeId>),
}
