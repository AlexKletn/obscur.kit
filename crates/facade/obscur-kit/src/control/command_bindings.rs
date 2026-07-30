//! [`CommandBindings`] — per-node command registrations that follow their nodes.

use std::cell::RefCell;
use std::rc::Rc;

use crate::headless::input::{SelectorResolver, SelectorTarget};
use crate::identity::{InternedSymbol, NodeId};
use crate::interaction::event::{ListenerPhase, UiEvent};
use crate::runtime::{CommandHandler, UiRuntime};
use crate::view::view::{AUTOMATION_ID, TEST_ID, ViewValue};

/// A handler held so it can be re-registered at a different node without being
/// consumed.
///
/// [`UiRuntime::on_command_at`] takes the handler by value, and a node id is not
/// stable across a structural change, so a registration that has to *move* cannot
/// hand the closure over directly. What is registered is a trampoline over this
/// shared cell; re-binding installs a new trampoline over the same handler.
pub(crate) type SharedHandler = Rc<RefCell<CommandHandler>>;

/// One positional registration: what to run, for which command, at whichever node
/// `target` currently resolves to.
struct Binding {
    /// How the node is named — a `test-id`, an `automation-id`, or a direct id
    /// (spec §7.5).
    target: SelectorTarget,
    /// The command symbol this registration answers.
    command: InternedSymbol,
    /// `None` for a §2.6 handler (link 1/3 of the routing chain), `Some(phase)` for
    /// a §2.3 listener in that phase.
    phase: Option<ListenerPhase>,
    /// The application's handler, shared so a re-bind does not consume it.
    handler: SharedHandler,
    /// The node this is currently registered at, if it has resolved yet.
    bound: Option<NodeId>,
}

/// The table of positional command registrations an application asked for, and the
/// nodes they are currently attached to (spec §2.6 links 1 and 3, §2.3 listeners).
///
/// # Why registrations are named, not passed as node ids
///
/// §2.6 routes a command *focused node → focus scope → ancestors → window →
/// application*, and [`UiRuntime`] implements all five — but the first four are
/// keyed by [`NodeId`], and a node id does not exist until a frame has built the
/// tree. An application configuring itself before `run` therefore has no id to
/// register with, which is exactly why only link 5 was reachable from `App`.
///
/// A [`SelectorTarget`] is the identifier that *does* exist beforehand: §7.5's
/// stable `test-id` / `automation-id`, written on the markup itself and resolved by
/// [`SelectorResolver`] against the live tree. So the application names the node,
/// this table resolves the name after every commit, and the registration lands on
/// whichever node currently carries it.
///
/// # Following the *name* across frames
///
/// A resolved binding is left alone while its node still answers to the name:
/// re-resolving per frame would cost a tree walk per registration, and
/// re-registering would be pointless for a handler (it replaces) and a leak for a
/// listener (they accumulate). It is released and resolved again when the node stops
/// answering — either it left the tree, or it is still there and no longer carries
/// the identifier, which is what happens when a reconcile patches an element in
/// place instead of replacing it.
///
/// That is the whole reason the check is [`answers_to`] and not merely
/// "is this id still live": the name is the contract, the node id is an
/// implementation detail of one frame's tree, and a handler must not keep answering
/// for an element that gave the name up. Confirming a *known* node still carries a
/// name is one node lookup; finding which node carries it is a tree walk, so this
/// pays the walk only when something actually changed. `NodeId`s are generational,
/// so a stale id can never alias a new node either.
#[derive(Default)]
pub(crate) struct CommandBindings {
    entries: Vec<Binding>,
}

impl CommandBindings {
    /// Adds a §2.6 per-node handler for `command` at `target`.
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

    /// Adds a §2.3 listener for `command` at `target`, running in `phase`.
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

    /// Resolves every unbound (or released) registration against `runtime`'s
    /// current tree and registers it, returning how many were newly bound.
    ///
    /// Idempotent: called after every commit, it does nothing at all once every
    /// name has resolved and its node still answers to it. A name that resolves to
    /// nothing — markup that is not on screen yet, or a `test-id` that was never
    /// authored — stays pending and is retried on the next commit rather than
    /// failing; a coordinate target never resolves to a node and therefore never
    /// binds (see [`SelectorResolver`]).
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

/// Whether `node` is still the node `target` names.
///
/// The cheap half of resolution: [`SelectorResolver`] answers "which node carries
/// this name" by walking the tree, and this answers "does *this* node still carry
/// it" by reading one node's properties. A stable identifier lives in the retained
/// property list rather than in the style attributes — that is what keeps §7.5's "no
/// effect on matching, layout or visuals" true — so this reads it from there, exactly
/// as the resolver does.
///
/// A [`Coordinate`](SelectorTarget::Coordinate) never names a node (hit-testing is
/// the runtime's job on the real input path), so it never answers.
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
