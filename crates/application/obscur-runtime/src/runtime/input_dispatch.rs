//! [`InputDispatch`]: what dispatching one queued [`InputEvent`] produced.

use obscur_identity::NodeId;

use super::{KeyDispatch, PointerDispatch};

#[doc = include_str!("../../docs/api/runtime/input_dispatch.doc/InputDispatch.md")]
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
