use obscur_identity::NodeId;

#[doc = include_str!("../../docs/api/request/activate_window.doc/FocusRequest.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusRequest {
    /// Activate the window without changing which element holds focus.
    WindowOnly,
    /// Activate the window and move focus to the identified element.
    Element(NodeId),
}

impl FocusRequest {
    /// Returns the target element for an [`FocusRequest::Element`] request, else
    /// `None`.
    #[must_use]
    pub const fn element(self) -> Option<NodeId> {
        match self {
            Self::Element(node) => Some(node),
            Self::WindowOnly => None,
        }
    }
}
