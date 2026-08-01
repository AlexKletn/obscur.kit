#![doc = include_str!("key_dispatch.doc/_module.md")]

use obscur_identity::{InternedSymbol, NodeId};

#[doc = include_str!("key_dispatch.doc/KeyDispatch.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyDispatch {
    #[doc = include_str!("key_dispatch.doc/target.md")]
    target: Option<NodeId>,
    #[doc = include_str!("key_dispatch.doc/command.md")]
    command: Option<InternedSymbol>,
}

impl KeyDispatch {
    #[doc = include_str!("key_dispatch.doc/new.md")]
    #[must_use]
    pub const fn new(target: Option<NodeId>, command: Option<InternedSymbol>) -> Self {
        Self { target, command }
    }

    #[doc = include_str!("key_dispatch.doc/unhandled.md")]
    #[must_use]
    pub const fn unhandled() -> Self {
        Self {
            target: None,
            command: None,
        }
    }

    #[doc = include_str!("key_dispatch.doc/target_2.md")]
    #[must_use]
    pub const fn target(&self) -> Option<NodeId> {
        self.target
    }

    #[doc = include_str!("key_dispatch.doc/command_2.md")]
    #[must_use]
    pub const fn command(&self) -> Option<InternedSymbol> {
        self.command
    }

    #[doc = include_str!("key_dispatch.doc/is_handled.md")]
    #[must_use]
    pub const fn is_handled(&self) -> bool {
        self.target.is_some()
    }

    #[doc = include_str!("key_dispatch.doc/resolved_command.md")]
    #[must_use]
    pub const fn resolved_command(&self) -> bool {
        self.command.is_some()
    }
}
