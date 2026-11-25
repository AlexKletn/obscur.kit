//! [`HandlerRegistry`]: what is bound where, for §2.3 propagation and §2.6 routing.

use std::collections::HashMap;

use obscur_identity::{FastMap, InternedSymbol, NodeId};
use obscur_interaction::command::{
    Command, CommandRegistry, CommandRouter, CommandState, Shortcut,
};
use obscur_interaction::event::{EventPhase, ListenerPhase, UiEvent};
use obscur_interaction::focus::{FocusRestore, FocusScope, FocusScopeId};

use super::{AppCommands, CommandHandler, CommandTarget, EventListener, IntoCommandHandler};

/// The shape both editable-field registries take: a command symbol to the
/// closure the application registered for it, called with the field's text.
type TextHandlers = FastMap<InternedSymbol, Box<dyn FnMut(&str)>>;

#[doc = include_str!("../../docs/api/runtime/handler_registry.doc/HandlerRegistry.md")]
pub(crate) struct HandlerRegistry {
    #[doc = include_str!("../../docs/api/runtime/handler_registry.doc/handlers.md")]
    handlers: FastMap<InternedSymbol, CommandHandler>,
    /// §2.6 links 1 and 3: handlers bound at a specific node, which is what makes
    /// routing positional — the same symbol runs different logic depending on
    /// where the command originated.
    node_handlers: FastMap<(NodeId, InternedSymbol), CommandHandler>,
    /// §2.3 listeners, keyed by the node they observe on. Several per key, run in
    /// registration order — unlike a handler, a listener never replaces an
    /// earlier one.
    listeners: FastMap<(NodeId, InternedSymbol), Vec<EventListener>>,
    /// The open focus-containment scopes, as a stack: the last opened is the
    /// innermost, which is the one §2.6 link 2 consults first.
    scopes: Vec<FocusScope>,
    #[doc = include_str!("../../docs/api/runtime/handler_registry.doc/focus_restore.md")]
    focus_restore: FocusRestore,
    #[doc = include_str!("../../docs/api/runtime/handler_registry.doc/declared_traps.md")]
    declared_traps: Vec<FocusScopeId>,
    /// §2.6 link 2: handlers bound to a focus scope, dropped with the scope.
    scope_handlers: HashMap<(FocusScopeId, InternedSymbol), CommandHandler>,
    /// §2.6 link 5: the shared application-level fallback. A handle, so several
    /// windows' runtimes given the same one share a genuinely application-wide
    /// level rather than each keeping a second per-window table.
    app_commands: AppCommands,
    /// §6.7 per-keystroke handlers for the focused editable's `on-input`
    /// command, called with the field's whole resulting text.
    text_input_handlers: TextHandlers,
    /// §6.7 commit handlers for the focused editable's `on-change` command,
    /// called once per edit session with the committed text. A distinct table
    /// from [`text_input_handlers`](Self::text_input_handlers) so an application
    /// can register both on one field without conflation.
    change_handlers: TextHandlers,
    #[doc = include_str!("../../docs/api/runtime/handler_registry.doc/value_handlers.md")]
    value_handlers: FastMap<InternedSymbol, Box<dyn FnMut(f32, f32)>>,
    /// The shortcut → command-symbol table keyboard input resolves against.
    commands: CommandRouter,
    /// The §6.7 unified action descriptors and their live [`CommandState`], which
    /// is what makes one `set_state` call disable a shortcut, a menu item and a
    /// click at once.
    command_registry: CommandRegistry,
}

impl HandlerRegistry {
    /// An empty registry: nothing bound anywhere, no scope open.
    pub(crate) fn new() -> Self {
        Self {
            handlers: HashMap::default(),
            node_handlers: HashMap::default(),
            listeners: HashMap::default(),
            scopes: Vec::new(),
            focus_restore: FocusRestore::new(),
            declared_traps: Vec::new(),
            scope_handlers: HashMap::default(),
            app_commands: AppCommands::new(),
            text_input_handlers: TextHandlers::default(),
            change_handlers: TextHandlers::default(),
            value_handlers: HashMap::default(),
            commands: CommandRouter::new(),
            command_registry: CommandRegistry::new(),
        }
    }

    // ---- §2.6 link 4: the window surface ------------------------------------

    /// Binds `handler` to `command` at the window level, replacing any previous.
    pub(crate) fn set_command<M>(
        &mut self,
        command: InternedSymbol,
        handler: impl IntoCommandHandler<M>,
    ) {
        self.handlers
            .insert(command, handler.into_command_handler());
    }

    /// Whether a window-level handler is bound to `command`.
    pub(crate) fn has_command(&self, command: InternedSymbol) -> bool {
        self.handlers.contains_key(&command)
    }

    #[doc = include_str!("../../docs/api/runtime/handler_registry.doc/remove_command.md")]
    pub(crate) fn remove_command(&mut self, command: InternedSymbol) -> bool {
        self.handlers.remove(&command).is_some()
    }

    // ---- §2.6 links 1 and 3: nodes ------------------------------------------

    /// Binds `handler` to `command` at `node`, replacing any previous.
    pub(crate) fn set_node_command<M>(
        &mut self,
        node: NodeId,
        command: InternedSymbol,
        handler: impl IntoCommandHandler<M>,
    ) {
        self.node_handlers
            .insert((node, command), handler.into_command_handler());
    }

    /// Whether a handler is bound to `command` at `node`.
    pub(crate) fn has_node_command(&self, node: NodeId, command: InternedSymbol) -> bool {
        self.node_handlers.contains_key(&(node, command))
    }

    #[doc = include_str!("../../docs/api/runtime/handler_registry.doc/clear_node.md")]
    pub(crate) fn clear_node(&mut self, node: NodeId) -> usize {
        let before = self.node_handlers.len();
        self.node_handlers.retain(|(at, _), _| *at != node);
        let handlers = before - self.node_handlers.len();

        let mut listeners = 0;
        self.listeners.retain(|(at, _), registered| {
            if *at == node {
                listeners += registered.len();
                false
            } else {
                true
            }
        });
        handlers + listeners
    }

    // ---- §2.3 listeners ------------------------------------------------------

    /// Adds a listener for `command` on `node`, running in `phase`. Adds — never
    /// replaces: several listeners per key run in registration order.
    pub(crate) fn add_listener<M>(
        &mut self,
        node: NodeId,
        command: InternedSymbol,
        phase: ListenerPhase,
        listener: impl IntoCommandHandler<M>,
    ) {
        self.listeners
            .entry((node, command))
            .or_default()
            .push(EventListener::new(phase, listener.into_command_handler()));
    }

    /// How many listeners are registered for `command` on `node`.
    pub(crate) fn listener_count(&self, node: NodeId, command: InternedSymbol) -> usize {
        self.listeners.get(&(node, command)).map_or(0, Vec::len)
    }

    #[doc = include_str!("../../docs/api/runtime/handler_registry.doc/has_listeners.md")]
    pub(crate) fn has_listeners(&self) -> bool {
        !self.listeners.is_empty()
    }

    #[doc = include_str!("../../docs/api/runtime/handler_registry.doc/run_listeners.md")]
    pub(crate) fn run_listeners(
        &mut self,
        node: NodeId,
        command: InternedSymbol,
        phase: EventPhase,
        event: &mut UiEvent,
    ) -> usize {
        let Some(listeners) = self.listeners.get_mut(&(node, command)) else {
            return 0;
        };
        let mut run = 0;
        for listener in listeners.iter_mut() {
            if event.is_immediate_stopped() {
                break;
            }
            if !listener.runs_in(phase) {
                continue;
            }
            listener.handler_mut().call(event);
            run += 1;
        }
        run
    }

    // ---- §2.6 link 2: focus scopes -------------------------------------------

    /// Opens `scope` as the innermost scope. Re-opening the same
    /// [`FocusScopeId`] replaces its membership rather than stacking a duplicate.
    pub(crate) fn open_scope(&mut self, scope: FocusScope) {
        let id = scope.id();
        self.scopes.retain(|open| open.id() != id);
        self.scopes.push(scope);
    }

    /// Closes the scope with this `id` **and drops its handlers**, returning
    /// whether one was open. The two go together: a closed dialog whose handlers
    /// survived would keep claiming commands from outside itself.
    pub(crate) fn close_scope(&mut self, id: FocusScopeId) -> bool {
        let before = self.scopes.len();
        self.scopes.retain(|scope| scope.id() != id);
        let closed = self.scopes.len() != before;
        if closed {
            self.scope_handlers.retain(|(at, _), _| *at != id);
        }
        closed
    }

    /// The open scopes, outermost first.
    pub(crate) fn scopes(&self) -> &[FocusScope] {
        &self.scopes
    }

    /// The open scope with this id, if one is open.
    pub(crate) fn scope(&self, id: FocusScopeId) -> Option<&FocusScope> {
        self.scopes.iter().find(|scope| scope.id() == id)
    }

    /// Records what focus a scope displaced as it opened.
    pub(crate) fn save_focus_for_scope(&mut self, previously_focused: Option<NodeId>) {
        self.focus_restore.save(previously_focused);
    }

    #[doc = include_str!("../../docs/api/runtime/handler_registry.doc/restore_focus_for_scope.md")]
    pub(crate) fn restore_focus_for_scope(&mut self) -> Option<NodeId> {
        self.focus_restore.restore()
    }

    /// The scopes of the traps the previous reconciliation saw.
    pub(crate) fn declared_traps(&self) -> &[FocusScopeId] {
        &self.declared_traps
    }

    /// Replaces the remembered trap scopes with this frame's answer.
    pub(crate) fn set_declared_traps(&mut self, traps: Vec<FocusScopeId>) {
        self.declared_traps = traps;
    }

    /// Binds `handler` to `command` on scope `id`, replacing any previous.
    pub(crate) fn set_scope_command<M>(
        &mut self,
        id: FocusScopeId,
        command: InternedSymbol,
        handler: impl IntoCommandHandler<M>,
    ) {
        self.scope_handlers
            .insert((id, command), handler.into_command_handler());
    }

    /// Whether a handler is bound to `command` on scope `id`.
    pub(crate) fn has_scope_command(&self, id: FocusScopeId, command: InternedSymbol) -> bool {
        self.scope_handlers.contains_key(&(id, command))
    }

    // ---- §2.6 link 5: the application fallback -------------------------------

    /// Installs the shared application-level fallback.
    pub(crate) fn set_app_commands(&mut self, commands: AppCommands) {
        self.app_commands = commands;
    }

    /// A handle to the application fallback. The handle *shares* the registry, so
    /// registering through it reaches this runtime and every other one given the
    /// same handle.
    pub(crate) fn app_commands(&self) -> AppCommands {
        self.app_commands.clone()
    }

    // ---- the route walk ------------------------------------------------------

    #[doc = include_str!("../../docs/api/runtime/handler_registry.doc/dispatch_along.md")]
    pub(crate) fn dispatch_along(
        &mut self,
        route: &[CommandTarget],
        command: InternedSymbol,
        event: &mut UiEvent,
    ) -> bool {
        for (index, target) in route.iter().enumerate() {
            // A listener that stopped propagation stops the chain too: the chain
            // is the outward leg of the same propagation, not a second mechanism.
            if event.is_stopped() {
                return false;
            }
            event.phase = if index == 0 && matches!(target, CommandTarget::Node(_)) {
                EventPhase::Target
            } else {
                EventPhase::Bubble
            };
            event.current_target = match target {
                CommandTarget::Node(node) => Some(*node),
                _ => None,
            };
            let claimed = match target {
                CommandTarget::Node(node) => match self.node_handlers.get_mut(&(*node, command)) {
                    Some(handler) => {
                        handler.call(event);
                        true
                    }
                    None => false,
                },
                CommandTarget::Scope(scope) => {
                    match self.scope_handlers.get_mut(&(*scope, command)) {
                        Some(handler) => {
                            handler.call(event);
                            true
                        }
                        None => false,
                    }
                }
                CommandTarget::Window => match self.handlers.get_mut(&command) {
                    Some(handler) => {
                        handler.call(event);
                        true
                    }
                    None => false,
                },
                CommandTarget::App => self.app_commands.dispatch(command, event),
            };
            if claimed {
                return true;
            }
        }
        false
    }

    // ---- §6.7 editable-field handlers ----------------------------------------

    /// Binds the per-keystroke `on-input` handler for `command`.
    pub(crate) fn set_text_input(
        &mut self,
        command: InternedSymbol,
        handler: impl FnMut(&str) + 'static,
    ) {
        self.text_input_handlers.insert(command, Box::new(handler));
    }

    /// Whether a text-input handler is bound to `command`.
    pub(crate) fn has_text_input(&self, command: InternedSymbol) -> bool {
        self.text_input_handlers.contains_key(&command)
    }

    /// Runs the text-input handler for `command` (if any) with `text`, returning
    /// whether one ran. Asks for no frame — the caller decides that.
    /// Registers the handler a numeric control reports through.
    pub(crate) fn set_value_input(
        &mut self,
        command: InternedSymbol,
        handler: impl FnMut(f32, f32) + 'static,
    ) {
        self.value_handlers.insert(command, Box::new(handler));
    }

    /// Reports a numeric control's new value, and whether anyone was listening.
    pub(crate) fn dispatch_value_input(
        &mut self,
        command: InternedSymbol,
        low: f32,
        high: f32,
    ) -> bool {
        if let Some(handler) = self.value_handlers.get_mut(&command) {
            handler(low, high);
            true
        } else {
            false
        }
    }

    pub(crate) fn dispatch_text_input(&mut self, command: InternedSymbol, text: &str) -> bool {
        if let Some(handler) = self.text_input_handlers.get_mut(&command) {
            handler(text);
            true
        } else {
            false
        }
    }

    /// Binds the commit `on-change` handler for `command`.
    pub(crate) fn set_change(
        &mut self,
        command: InternedSymbol,
        handler: impl FnMut(&str) + 'static,
    ) {
        self.change_handlers.insert(command, Box::new(handler));
    }

    /// Whether a change (commit) handler is bound to `command`.
    pub(crate) fn has_change(&self, command: InternedSymbol) -> bool {
        self.change_handlers.contains_key(&command)
    }

    /// Runs the change handler for `command` (if any) with `text`, returning
    /// whether one ran. Asks for no frame — the caller decides that.
    pub(crate) fn dispatch_change(&mut self, command: InternedSymbol, text: &str) -> bool {
        if let Some(handler) = self.change_handlers.get_mut(&command) {
            handler(text);
            true
        } else {
            false
        }
    }

    // ---- §6.7 shortcuts and descriptors --------------------------------------

    /// Binds `shortcut` to `command` in the keyboard table.
    pub(crate) fn bind_shortcut(&mut self, shortcut: Shortcut, command: InternedSymbol) {
        self.commands.bind(shortcut, command);
    }

    /// The shortcut → command-symbol table keyboard input resolves against.
    pub(crate) const fn command_router(&self) -> &CommandRouter {
        &self.commands
    }

    /// Registers a unified [`Command`] descriptor and binds its shortcut if it
    /// declares one — the single registration that lights up the keyboard, the
    /// menu and the click together (spec §6.7).
    pub(crate) fn register_command(&mut self, command: Command) {
        if let Some(shortcut) = command.shortcut() {
            self.commands.bind(shortcut, command.id());
        }
        self.command_registry.register(command);
    }

    /// The unified action descriptors and their live state (spec §6.7).
    pub(crate) const fn command_registry(&self) -> &CommandRegistry {
        &self.command_registry
    }

    /// Sets the live [`CommandState`] of a registered command, returning whether
    /// one was registered under `command`.
    pub(crate) fn set_command_state(
        &mut self,
        command: InternedSymbol,
        state: CommandState,
    ) -> bool {
        self.command_registry.set_state(command, state)
    }

    /// Whether `command`'s registered state permits a dispatch. A symbol with no
    /// registered descriptor is unconstrained, so applications routing raw
    /// symbols are unaffected (spec §6.7).
    pub(crate) fn permits_dispatch(&self, command: InternedSymbol) -> bool {
        self.command_registry.permits_dispatch(command)
    }
}

impl std::fmt::Debug for HandlerRegistry {
    /// Counts what is bound, never the closures — a handler has no useful
    /// rendering.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HandlerRegistry")
            .field("handlers", &self.handlers.len())
            .field("node_handlers", &self.node_handlers.len())
            .field("listener_keys", &self.listeners.len())
            .field("scopes", &self.scopes.len())
            .field("scope_handlers", &self.scope_handlers.len())
            .field("text_input_handlers", &self.text_input_handlers.len())
            .field("change_handlers", &self.change_handlers.len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::HandlerRegistry;
    use obscur_identity::{InternedSymbol, NodeId};
    use obscur_interaction::event::{EventOrigin, ListenerPhase, UiEvent};
    use obscur_interaction::focus::{FocusScope, FocusScopeId};

    const SAVE: InternedSymbol = InternedSymbol::named("save");

    #[test]
    fn clearing_a_node_drops_its_handlers_and_its_listeners_together() {
        let mut registry = HandlerRegistry::new();
        let node = NodeId::from_raw(1);
        let other = NodeId::from_raw(2);
        registry.set_node_command(node, SAVE, || {});
        registry.add_listener(node, SAVE, ListenerPhase::Bubble, || {});
        registry.add_listener(node, SAVE, ListenerPhase::Capture, || {});
        registry.set_node_command(other, SAVE, || {});

        // Three registrations went — counted per listener, not per key — and the
        // other node kept its own.
        assert_eq!(registry.clear_node(node), 3);
        assert!(!registry.has_node_command(node, SAVE));
        assert_eq!(registry.listener_count(node, SAVE), 0);
        assert!(registry.has_node_command(other, SAVE));
    }

    #[test]
    fn closing_a_scope_drops_the_handlers_bound_to_it() {
        let mut registry = HandlerRegistry::new();
        let id = FocusScopeId::from_raw(7);
        registry.open_scope(FocusScope::new(id).with_member(NodeId::from_raw(1)));
        registry.set_scope_command(id, SAVE, || {});
        assert!(registry.has_scope_command(id, SAVE));

        assert!(registry.close_scope(id));
        // The handler goes with the scope: a closed dialog stops claiming.
        assert!(!registry.has_scope_command(id, SAVE));
        assert!(registry.scopes().is_empty());
        // Closing again reports that nothing was open.
        assert!(!registry.close_scope(id));
    }

    #[test]
    fn reopening_a_scope_replaces_it_rather_than_stacking_a_duplicate() {
        let mut registry = HandlerRegistry::new();
        let id = FocusScopeId::from_raw(3);
        registry.open_scope(FocusScope::new(id).with_member(NodeId::from_raw(1)));
        registry.open_scope(FocusScope::new(id).with_member(NodeId::from_raw(2)));
        assert_eq!(registry.scopes().len(), 1);
        assert!(registry.scopes()[0].contains(NodeId::from_raw(2)));
    }

    #[test]
    fn a_listener_is_added_never_replaced_and_a_handler_is_replaced_never_added() {
        let mut registry = HandlerRegistry::new();
        let node = NodeId::from_raw(1);
        registry.add_listener(node, SAVE, ListenerPhase::Bubble, || {});
        registry.add_listener(node, SAVE, ListenerPhase::Bubble, || {});
        assert_eq!(registry.listener_count(node, SAVE), 2);

        registry.set_node_command(node, SAVE, || {});
        registry.set_node_command(node, SAVE, || {});
        // Two registrations, one handler: `clear_node` reports 2 listeners + 1.
        assert_eq!(registry.clear_node(node), 3);
    }

    #[test]
    fn an_empty_registry_reports_no_listeners_so_the_walk_can_be_skipped() {
        let mut registry = HandlerRegistry::new();
        assert!(!registry.has_listeners());
        registry.add_listener(NodeId::from_raw(1), SAVE, ListenerPhase::Bubble, || {});
        assert!(registry.has_listeners());
    }

    #[test]
    fn a_stopped_event_claims_nothing_further_along_the_route() {
        use super::CommandTarget;
        let mut registry = HandlerRegistry::new();
        registry.set_command(SAVE, || {});
        let mut event = UiEvent::detached(EventOrigin::Synthetic);
        event.stop_propagation();
        // The window link would have claimed it; a stopped event never reaches it.
        assert!(!registry.dispatch_along(&[CommandTarget::Window], SAVE, &mut event));
    }
}
