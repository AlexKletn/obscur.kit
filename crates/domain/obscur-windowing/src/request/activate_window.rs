#![doc = include_str!("activate_window.doc/_module.md")]

use obscur_identity::{NodeId, WindowId};

#[doc = include_str!("activate_window.doc/FocusRequest.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusRequest {
    #[doc = include_str!("activate_window.doc/WindowOnly.md")]
    WindowOnly,
    #[doc = include_str!("activate_window.doc/Element.md")]
    Element(NodeId),
}

impl FocusRequest {
    #[doc = include_str!("activate_window.doc/element.md")]
    #[must_use]
    pub const fn element(self) -> Option<NodeId> {
        match self {
            Self::Element(node) => Some(node),
            Self::WindowOnly => None,
        }
    }
}

#[doc = include_str!("activate_window.doc/ActivationResult.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivationResult {
    #[doc = include_str!("activate_window.doc/Activated.md")]
    Activated,
    #[doc = include_str!("activate_window.doc/Denied.md")]
    Denied,
    #[doc = include_str!("activate_window.doc/Unsupported.md")]
    Unsupported,
    #[doc = include_str!("activate_window.doc/Failed.md")]
    Failed(Box<str>),
}

impl ActivationResult {
    #[doc = include_str!("activate_window.doc/is_activated.md")]
    #[must_use]
    pub const fn is_activated(&self) -> bool {
        matches!(self, Self::Activated)
    }
}

#[doc = include_str!("activate_window.doc/ActivateWindow.md")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivateWindow {
    #[doc = include_str!("activate_window.doc/target.md")]
    target: WindowId,
    #[doc = include_str!("activate_window.doc/focus.md")]
    focus: FocusRequest,
}

impl ActivateWindow {
    #[doc = include_str!("activate_window.doc/window.md")]
    #[must_use]
    pub const fn window(target: WindowId) -> Self {
        Self {
            target,
            focus: FocusRequest::WindowOnly,
        }
    }

    #[doc = include_str!("activate_window.doc/element_2.md")]
    #[must_use]
    pub const fn element(target: WindowId, node: NodeId) -> Self {
        Self {
            target,
            focus: FocusRequest::Element(node),
        }
    }

    #[doc = include_str!("activate_window.doc/target_2.md")]
    #[must_use]
    pub const fn target(&self) -> WindowId {
        self.target
    }

    #[doc = include_str!("activate_window.doc/focus_2.md")]
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
