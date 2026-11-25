//! The [`CommandTarget`] links of §2.6's command routing chain.

use obscur_identity::NodeId;
use obscur_interaction::focus::FocusScopeId;

#[doc = include_str!("../../docs/api/runtime/command_target.doc/CommandTarget.md")]
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
