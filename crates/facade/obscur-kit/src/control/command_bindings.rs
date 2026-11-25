//! [`CommandBindings`] — per-node command registrations that follow their nodes.

mod command_binding;

use command_binding::CommandBinding;
use std::cell::RefCell;
use std::rc::Rc;

use crate::headless::input::{SelectorResolver, SelectorTarget};
use crate::identity::{InternedSymbol, NodeId};
use crate::interaction::event::{ListenerPhase, UiEvent};
use crate::runtime::{CommandHandler, UiRuntime};
use crate::view::view::{AUTOMATION_ID, TEST_ID, ViewValue};

#[doc = include_str!("../../docs/api/control/command_bindings.doc/SharedHandler.md")]
pub(crate) type SharedHandler = Rc<RefCell<CommandHandler>>;

#[doc = include_str!("../../docs/api/control/command_bindings.doc/CommandBindings.md")]
#[derive(Default)]
pub(crate) struct CommandBindings {
    entries: Vec<CommandBinding>,
}

impl CommandBindings {
    /// Adds a §2.6 per-node handler for `command` at `target`.
    pub(crate) fn push_handler(
        &mut self,
        target: SelectorTarget,
        command: InternedSymbol,
        handler: SharedHandler,
    ) {
        self.entries.push(CommandBinding {
            target,
            command,
            phase: None,
            handler,
            bound: None,
        });
    }

    /// Adds a §2.3 listener for `command` at `target`, running in `phase`.
    pub(crate) fn push_listener(
        &mut self,
        target: SelectorTarget,
        command: InternedSymbol,
        phase: ListenerPhase,
        handler: SharedHandler,
    ) {
        self.entries.push(CommandBinding {
            target,
            command,
            phase: Some(phase),
            handler,
            bound: None,
        });
    }

    #[doc = include_str!("../../docs/api/control/command_bindings.doc/apply.md")]
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

    /// The node `target`'s registrations are currently attached to, if it has
    /// resolved. The first matching registration answers, which is enough: every
    /// registration for one target resolves to the same node.
    pub(crate) fn bound_node(&self, target: &SelectorTarget) -> Option<NodeId> {
        self.entries
            .iter()
            .find(|entry| &entry.target == target)
            .and_then(|entry| entry.bound)
    }

    /// How many registrations are waiting for their name to resolve.
    pub(crate) fn pending(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.bound.is_none())
            .count()
    }

    /// How many registrations the table holds in total.
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }
}

#[doc = include_str!("../../docs/api/control/command_bindings.doc/answers_to.md")]
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
        SelectorTarget::Semantic { .. } | SelectorTarget::Component(_) => {
            SelectorResolver::new().resolve(runtime, target) == Some(node)
        }
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
