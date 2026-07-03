//! Window activation and element focus requests (spec §6.3).

use obscur_identity::{NodeId, WindowId};

/// What a [`ActivateWindow`] request should focus (spec §6.3).
///
/// Spec §6.3 separates *window activation* from *element focus*. This enum is a
/// tightly-coupled part of [`ActivateWindow`]: it distinguishes activating the
/// window alone from also moving keyboard focus to a specific element, so it is
/// colocated here rather than given its own module.
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

/// The typed outcome of an [`ActivateWindow`] request (spec §6.3).
///
/// Spec §6.3 enumerates four possible results. Colocated here as the inseparable
/// result type of [`ActivateWindow`], mirroring the `Modality`/`WindowRelation`
/// precedent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivationResult {
    /// The window was activated (and any requested element focused).
    Activated,
    /// The platform declined the request, e.g. to prevent focus stealing.
    ///
    /// Per spec §6.3, focus-stealing restrictions are normal platform behavior,
    /// **not** a toolkit failure: a `Denied` result is an expected outcome an
    /// application should handle gracefully, not an error condition.
    Denied,
    /// The platform does not support programmatic activation/focus.
    Unsupported,
    /// Activation was attempted but failed at runtime, carrying a reason.
    Failed(Box<str>),
}

impl ActivationResult {
    /// Returns whether the request resulted in activation.
    #[must_use]
    pub const fn is_activated(&self) -> bool {
        matches!(self, Self::Activated)
    }
}

/// An application's request to activate a window and optionally focus an element
/// within it (spec §6.3).
///
/// A `ActivateWindow` names a `target` window and a [`FocusRequest`] describing
/// whether to also move keyboard focus to a specific element. It is a pure
/// command value; a backend consumes it and reports an [`ActivationResult`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivateWindow {
    /// The window to activate.
    target: WindowId,
    /// What to focus once the window is activated.
    focus: FocusRequest,
}

impl ActivateWindow {
    /// Requests activation of `target` without changing element focus.
    #[must_use]
    pub const fn window(target: WindowId) -> Self {
        Self {
            target,
            focus: FocusRequest::WindowOnly,
        }
    }

    /// Requests activation of `target` and focus of the element `node` within it.
    #[must_use]
    pub const fn element(target: WindowId, node: NodeId) -> Self {
        Self {
            target,
            focus: FocusRequest::Element(node),
        }
    }

    /// Returns the window this request targets.
    #[must_use]
    pub const fn target(&self) -> WindowId {
        self.target
    }

    /// Returns what this request should focus.
    #[must_use]
    pub const fn focus(&self) -> FocusRequest {
        self.focus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn win() -> WindowId {
        WindowId::from_raw(1)
    }

    #[test]
    fn window_constructor_focuses_window_only() {
        let req = ActivateWindow::window(win());
        assert_eq!(req.target(), win());
        assert_eq!(req.focus(), FocusRequest::WindowOnly);
        assert_eq!(req.focus().element(), None);
    }

    #[test]
    fn element_constructor_carries_node() {
        let node = NodeId::from_raw(42);
        let req = ActivateWindow::element(win(), node);
        assert_eq!(req.target(), win());
        assert_eq!(req.focus(), FocusRequest::Element(node));
        assert_eq!(req.focus().element(), Some(node));
    }

    #[test]
    fn activation_result_predicate() {
        assert!(ActivationResult::Activated.is_activated());
        assert!(!ActivationResult::Denied.is_activated());
        assert!(!ActivationResult::Unsupported.is_activated());
        assert!(!ActivationResult::Failed(Box::from("x")).is_activated());
    }
}
