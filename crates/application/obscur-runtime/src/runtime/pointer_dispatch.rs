#![doc = include_str!("pointer_dispatch.doc/_module.md")]

use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::event::EventTrust;

#[doc = include_str!("pointer_dispatch.doc/PointerDispatch.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PointerDispatch {
    #[doc = include_str!("pointer_dispatch.doc/target.md")]
    pub target: Option<NodeId>,
    #[doc = include_str!("pointer_dispatch.doc/hover_changed.md")]
    pub hover_changed: bool,
    #[doc = include_str!("pointer_dispatch.doc/focus_changed.md")]
    pub focus_changed: bool,
    #[doc = include_str!("pointer_dispatch.doc/command.md")]
    pub command: Option<InternedSymbol>,
    #[doc = include_str!("pointer_dispatch.doc/current_target.md")]
    pub current_target: Option<NodeId>,
    #[doc = include_str!("pointer_dispatch.doc/trust.md")]
    pub trust: EventTrust,
}

impl PointerDispatch {
    #[doc = include_str!("pointer_dispatch.doc/new.md")]
    #[must_use]
    pub const fn new(
        target: Option<NodeId>,
        hover_changed: bool,
        focus_changed: bool,
        command: Option<InternedSymbol>,
        current_target: Option<NodeId>,
        trust: EventTrust,
    ) -> Self {
        Self {
            target,
            hover_changed,
            focus_changed,
            command,
            current_target,
            trust,
        }
    }

    #[doc = include_str!("pointer_dispatch.doc/target_2.md")]
    #[must_use]
    pub const fn target(&self) -> Option<NodeId> {
        self.target
    }

    #[doc = include_str!("pointer_dispatch.doc/hover_changed_2.md")]
    #[must_use]
    pub const fn hover_changed(&self) -> bool {
        self.hover_changed
    }

    #[doc = include_str!("pointer_dispatch.doc/focus_changed_2.md")]
    #[must_use]
    pub const fn focus_changed(&self) -> bool {
        self.focus_changed
    }

    #[doc = include_str!("pointer_dispatch.doc/command_2.md")]
    #[must_use]
    pub const fn command(&self) -> Option<InternedSymbol> {
        self.command
    }

    #[doc = include_str!("pointer_dispatch.doc/current_target_2.md")]
    #[must_use]
    pub const fn current_target(&self) -> Option<NodeId> {
        self.current_target
    }

    #[doc = include_str!("pointer_dispatch.doc/trust_2.md")]
    #[must_use]
    pub const fn trust(&self) -> EventTrust {
        self.trust
    }

    #[doc = include_str!("pointer_dispatch.doc/is_trusted.md")]
    #[must_use]
    pub const fn is_trusted(&self) -> bool {
        self.trust.is_trusted()
    }

    #[doc = include_str!("pointer_dispatch.doc/changed.md")]
    #[must_use]
    pub const fn changed(&self) -> bool {
        self.hover_changed || self.focus_changed
    }
}
