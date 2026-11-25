//! [`ControlRequest`] — one queued instruction from application code to the runtime.

use crate::runtime::UiRuntime;

#[doc = include_str!("../../docs/api/control/control_request.doc/ControlRequest.md")]
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
