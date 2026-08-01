#![doc = include_str!("command_bindings.doc/_module.md")]

use std::cell::RefCell;
use std::rc::Rc;

use crate::headless::input::{SelectorResolver, SelectorTarget};
use crate::identity::{InternedSymbol, NodeId};
use crate::interaction::event::{ListenerPhase, UiEvent};
use crate::runtime::{CommandHandler, UiRuntime};
use crate::view::view::{AUTOMATION_ID, TEST_ID, ViewValue};

#[doc = include_str!("command_bindings.doc/SharedHandler.md")]
pub(crate) type SharedHandler = Rc<RefCell<CommandHandler>>;

#[doc = include_str!("command_bindings.doc/Binding.md")]
struct Binding {
    #[doc = include_str!("command_bindings.doc/target.md")]
    target: SelectorTarget,
    #[doc = include_str!("command_bindings.doc/command.md")]
    command: InternedSymbol,
    #[doc = include_str!("command_bindings.doc/phase.md")]
    phase: Option<ListenerPhase>,
    #[doc = include_str!("command_bindings.doc/handler.md")]
    handler: SharedHandler,
    #[doc = include_str!("command_bindings.doc/bound.md")]
    bound: Option<NodeId>,
}

#[doc = include_str!("command_bindings.doc/CommandBindings.md")]
#[derive(Default)]
pub(crate) struct CommandBindings {
    entries: Vec<Binding>,
}

impl CommandBindings {
    #[doc = include_str!("command_bindings.doc/push_handler.md")]
    pub(crate) fn push_handler(
        &mut self,
        target: SelectorTarget,
        command: InternedSymbol,
        handler: SharedHandler,
    ) {
        self.entries.push(Binding {
            target,
            command,
            phase: None,
            handler,
            bound: None,
        });
    }

    #[doc = include_str!("command_bindings.doc/push_listener.md")]
    pub(crate) fn push_listener(
        &mut self,
        target: SelectorTarget,
        command: InternedSymbol,
        phase: ListenerPhase,
        handler: SharedHandler,
    ) {
        self.entries.push(Binding {
            target,
            command,
            phase: Some(phase),
            handler,
            bound: None,
        });
    }

    #[doc = include_str!("command_bindings.doc/apply.md")]
    pub(crate) fn apply(&mut self, runtime: &mut UiRuntime) -> usize {
        let mut newly_bound = 0;
        for entry in &mut self.entries {
            if entry
                .bound
                .is_some_and(|node| answers_to(runtime, node, &entry.target))
            {
                continue;
            }
            // The node stopped answering to the name. Its registrations must not fire
            // for it any more, and they are not reaped automatically, so drop them
            // before this name is bound elsewhere.
            if let Some(stale) = entry.bound.take() {
                runtime.clear_node_commands(stale);
            }
            let Some(node) = SelectorResolver::new().resolve(runtime, &entry.target) else {
                continue;
            };
            let handler = Rc::clone(&entry.handler);
            let trampoline = move |event: &mut UiEvent| handler.borrow_mut().call(event);
            match entry.phase {
                None => runtime.on_command_at(node, entry.command, trampoline),
                Some(phase) => runtime.add_event_listener(node, entry.command, phase, trampoline),
            }
            entry.bound = Some(node);
            newly_bound += 1;
        }
        newly_bound
    }

    #[doc = include_str!("command_bindings.doc/bound_node.md")]
    pub(crate) fn bound_node(&self, target: &SelectorTarget) -> Option<NodeId> {
        self.entries
            .iter()
            .find(|entry| &entry.target == target)
            .and_then(|entry| entry.bound)
    }

    #[doc = include_str!("command_bindings.doc/pending.md")]
    pub(crate) fn pending(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.bound.is_none())
            .count()
    }

    #[doc = include_str!("command_bindings.doc/len.md")]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
}

#[doc = include_str!("command_bindings.doc/answers_to.md")]
fn answers_to(runtime: &UiRuntime, node: NodeId, target: &SelectorTarget) -> bool {
    let Some(retained) = runtime.tree().get(node) else {
        return false;
    };
    let carries = |attribute: InternedSymbol, id: &str| {
        retained.props().iter().any(|prop| {
            prop.name() == attribute
                && matches!(prop.value(), ViewValue::Text(text) if &**text == id)
        })
    };
    match target {
        SelectorTarget::Node(id) => *id == node,
        SelectorTarget::TestId(id) => carries(TEST_ID, id),
        SelectorTarget::AutomationId(id) => carries(AUTOMATION_ID, id),
        SelectorTarget::Coordinate(_) => false,
    }
}

impl std::fmt::Debug for CommandBindings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandBindings")
            .field("registrations", &self.entries.len())
            .field("pending", &self.pending())
            .finish()
    }
}
