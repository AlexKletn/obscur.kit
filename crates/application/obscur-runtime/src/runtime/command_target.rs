//! The [`CommandTarget`] links of §2.6's command routing chain.

use obscur_identity::NodeId;
use obscur_interaction::focus::FocusScopeId;

/// One link of the §2.6 command routing chain.
///
/// The spec's chain is *"focused node; focus scope; ancestors; window;
/// application fallback"*. A `CommandTarget` names one of those levels for a
/// concrete dispatch, and
/// [`UiRuntime::command_route`](super::UiRuntime::command_route) returns them in
/// the spec's order. A dispatch offers the command to each link in turn and
/// stops at the first that has a handler registered for it — the first link to
/// *claim* the command handles it, and no later link sees it.
///
/// [`Node`](Self::Node) covers **two** of the spec's five levels: the route's
/// first `Node` is the focused node (level 1), and every `Node` after the
/// [`Scope`](Self::Scope) links is an ancestor (level 3). They are the same kind
/// of link — a handler registered at a node — so they share a variant; their
/// *order* in the route is what distinguishes them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandTarget {
    /// A node in the tree: the focused node itself (level 1) or one of its
    /// ancestors (level 3), depending on its position in the route.
    Node(NodeId),
    /// A focus scope containing the focused node (level 2), innermost first.
    Scope(FocusScopeId),
    /// The window surface this runtime *is* (level 4) — the handlers registered
    /// by [`on_command`](super::UiRuntime::on_command).
    Window,
    /// The application fallback (level 5) — the
    /// [`AppCommands`](super::AppCommands) registry shared across windows.
    App,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_compare_by_level_and_identity() {
        let a = CommandTarget::Node(NodeId::from_raw(1));
        assert_eq!(a, CommandTarget::Node(NodeId::from_raw(1)));
        assert_ne!(a, CommandTarget::Node(NodeId::from_raw(2)));
        assert_ne!(a, CommandTarget::Window);
        assert_ne!(CommandTarget::Window, CommandTarget::App);
        assert_eq!(
            CommandTarget::Scope(FocusScopeId::from_raw(3)),
            CommandTarget::Scope(FocusScopeId::from_raw(3))
        );
    }
}
