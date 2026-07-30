//! [`ControlRequest`] — one queued instruction from application code to the runtime.

use crate::runtime::UiRuntime;

/// Something an application asked the runtime to do, waiting for the point in the
/// frame where doing it is safe.
///
/// Only the requests that need the *host's* collaborators are queued: replacing the
/// authored stylesheets needs the [`CssPipeline`](crate::bootstrap::CssPipeline)
/// that owns §4.12's parse cache, and an arbitrary mutation needs a `&mut
/// UiRuntime`. A positional registration is not here — it needs no runtime to be
/// *recorded*, only to be *resolved*, so it goes straight into
/// [`CommandBindings`](super::CommandBindings) and is resolved on the same pass.
pub(crate) enum ControlRequest {
    /// Replace the authored stylesheet set with these sources, in order.
    Stylesheets(Vec<String>),
    /// Run this once against the runtime.
    Runtime(Box<dyn FnOnce(&mut UiRuntime)>),
    /// Schedule a frame, for a state change the runtime cannot have noticed.
    Frame,
}

impl std::fmt::Debug for ControlRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stylesheets(sheets) => write!(f, "Stylesheets({} sources)", sheets.len()),
            Self::Runtime(_) => f.write_str("Runtime(..)"),
            Self::Frame => f.write_str("Frame"),
        }
    }
}
