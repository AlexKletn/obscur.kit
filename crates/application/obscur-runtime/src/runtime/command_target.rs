#![doc = include_str!("command_target.doc/_module.md")]

use obscur_identity::NodeId;
use obscur_interaction::focus::FocusScopeId;

#[doc = include_str!("command_target.doc/CommandTarget.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandTarget {
    #[doc = include_str!("command_target.doc/Node.md")]
    Node(NodeId),
    #[doc = include_str!("command_target.doc/Scope.md")]
    Scope(FocusScopeId),
    #[doc = include_str!("command_target.doc/Window.md")]
    Window,
    #[doc = include_str!("command_target.doc/App.md")]
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
