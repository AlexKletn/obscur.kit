//! Window activation and element focus requests (spec §6.3).

use obscur_identity::{NodeId, WindowId};

use super::FocusRequest;

#[doc = include_str!("../../docs/api/request/activate_window.doc/ActivateWindow.md")]
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
}
