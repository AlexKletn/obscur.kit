#![doc = include_str!("control_request.doc/_module.md")]

use crate::runtime::UiRuntime;

#[doc = include_str!("control_request.doc/ControlRequest.md")]
pub(crate) enum ControlRequest {
    #[doc = include_str!("control_request.doc/Stylesheets.md")]
    Stylesheets(Vec<String>),
    #[doc = include_str!("control_request.doc/Runtime.md")]
    Runtime(Box<dyn FnOnce(&mut UiRuntime)>),
    #[doc = include_str!("control_request.doc/Frame.md")]
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
