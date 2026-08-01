#![doc = include_str!("command_routing.doc/_module.md")]

use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::command::{
    Command, CommandRegistry, CommandRouter, CommandState, Shortcut,
};
use obscur_interaction::event::{EventOrigin, UiEvent};
use obscur_interaction::focus::{FocusScope, FocusScopeId};

use crate::frame::FrameReason;
use crate::runtime::{
    AppCommands, CommandDispatch, CommandTarget, EditableField, IntoCommandHandler,
    StructuralStates,
};

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("command_routing.doc/register_command.md")]
    pub fn register_command(&mut self, command: Command) {
        self.routing.register_command(command);
    }

    #[doc = include_str!("command_routing.doc/command_registry.md")]
    #[must_use]
    pub const fn command_registry(&self) -> &CommandRegistry {
        self.routing.command_registry()
    }

    #[doc = include_str!("command_routing.doc/bind_command.md")]
    pub fn bind_command(&mut self, shortcut: Shortcut, command: InternedSymbol) {
        self.routing.bind_shortcut(shortcut, command);
    }

    #[doc = include_str!("command_routing.doc/command_router.md")]
    #[must_use]
    pub const fn command_router(&self) -> &CommandRouter {
        self.routing.command_router()
    }

    #[doc = include_str!("command_routing.doc/set_command_state.md")]
    pub fn set_command_state(&mut self, command: InternedSymbol, state: CommandState) -> bool {
        self.routing.set_command_state(command, state)
    }

    #[doc = include_str!("command_routing.doc/on_command.md")]
    pub fn on_command<M>(&mut self, command: InternedSymbol, handler: impl IntoCommandHandler<M>) {
        self.routing.set_command(command, handler);
    }

    #[doc = include_str!("command_routing.doc/has_command_handler.md")]
    #[must_use]
    pub fn has_command_handler(&self, command: InternedSymbol) -> bool {
        self.routing.has_command(command)
    }

    #[doc = include_str!("command_routing.doc/on_command_at.md")]
    pub fn on_command_at<M>(
        &mut self,
        node: NodeId,
        command: InternedSymbol,
        handler: impl IntoCommandHandler<M>,
    ) {
        self.routing.set_node_command(node, command, handler);
    }

    #[doc = include_str!("command_routing.doc/has_node_command_handler.md")]
    #[must_use]
    pub fn has_node_command_handler(&self, node: NodeId, command: InternedSymbol) -> bool {
        self.routing.has_node_command(node, command)
    }

    #[doc = include_str!("command_routing.doc/clear_node_commands.md")]
    pub fn clear_node_commands(&mut self, node: NodeId) -> usize {
        self.routing.clear_node(node)
    }

    #[doc = include_str!("command_routing.doc/open_focus_scope.md")]
    pub fn open_focus_scope(&mut self, scope: FocusScope) {
        self.routing.open_scope(scope);
    }

    #[doc = include_str!("command_routing.doc/close_focus_scope.md")]
    pub fn close_focus_scope(&mut self, id: FocusScopeId) -> bool {
        self.routing.close_scope(id)
    }

    #[doc = include_str!("command_routing.doc/focus_scopes.md")]
    #[must_use]
    pub fn focus_scopes(&self) -> &[FocusScope] {
        self.routing.scopes()
    }

    #[doc = include_str!("command_routing.doc/on_scope_command.md")]
    pub fn on_scope_command<M>(
        &mut self,
        id: FocusScopeId,
        command: InternedSymbol,
        handler: impl IntoCommandHandler<M>,
    ) {
        self.routing.set_scope_command(id, command, handler);
    }

    #[doc = include_str!("command_routing.doc/has_scope_command_handler.md")]
    #[must_use]
    pub fn has_scope_command_handler(&self, id: FocusScopeId, command: InternedSymbol) -> bool {
        self.routing.has_scope_command(id, command)
    }

    #[doc = include_str!("command_routing.doc/set_app_commands.md")]
    pub fn set_app_commands(&mut self, commands: AppCommands) {
        self.routing.set_app_commands(commands);
    }

    #[doc = include_str!("command_routing.doc/app_commands.md")]
    #[must_use]
    pub fn app_commands(&self) -> AppCommands {
        self.routing.app_commands()
    }

    #[doc = include_str!("command_routing.doc/dispatch_command.md")]
    pub fn dispatch_command(&mut self, command: InternedSymbol) -> bool {
        self.dispatch_command_from(self.focused, command)
    }

    #[doc = include_str!("command_routing.doc/command_route.md")]
    #[must_use]
    pub fn command_route(&self, origin: Option<NodeId>) -> Vec<CommandTarget> {
        CommandDispatch::route(&self.tree, &self.routing, origin)
    }

    #[doc = include_str!("command_routing.doc/activate.md")]
    pub(super) fn activate(&mut self, node: NodeId, envelope: UiEvent) -> bool {
        // A disabled control is inert: it fires no activation command, whether the
        // activation came from a pointer click or an Enter/Space keypress.
        if StructuralStates::of(&self.tree).is_within_disabled(node) {
            return false;
        }
        let editable = EditableField::editable_ancestor(&self.tree, node).is_some();
        match CommandDispatch::activation_command_for(&self.tree, node, editable) {
            // The command originates at the ACTIVATED node, not at whatever is
            // focused: a pointer click on a non-focusable control must route from
            // the control the user hit (§2.6's chain, rooted at the origin). For
            // an Enter/Space activation `node` *is* the focused node, so both
            // callers get the spec's route.
            Some((_current_target, command)) => {
                self.dispatch_command_event(Some(node), command, envelope)
            }
            None => false,
        }
    }

    #[doc = include_str!("command_routing.doc/dispatch_command_from.md")]
    pub(super) fn dispatch_command_from(
        &mut self,
        origin: Option<NodeId>,
        command: InternedSymbol,
    ) -> bool {
        self.dispatch_command_event(origin, command, UiEvent::detached(EventOrigin::Synthetic))
    }

    #[doc = include_str!("command_routing.doc/dispatch_command_event.md")]
    pub(super) fn dispatch_command_event(
        &mut self,
        origin: Option<NodeId>,
        command: InternedSymbol,
        envelope: UiEvent,
    ) -> bool {
        let outcome =
            CommandDispatch::new(&self.tree, &mut self.routing).dispatch(origin, command, envelope);
        if outcome.anything_ran {
            self.request_frame(FrameReason::Input);
        }
        outcome.claimed
    }

    #[doc = include_str!("command_routing.doc/dispatch_text_input.md")]
    pub(super) fn dispatch_text_input(&mut self, command: InternedSymbol, text: &str) -> bool {
        let ran = self.routing.dispatch_text_input(command, text);
        if ran {
            self.request_frame(FrameReason::Input);
        }
        ran
    }

    #[doc = include_str!("command_routing.doc/dispatch_change.md")]
    pub(super) fn dispatch_change(&mut self, command: InternedSymbol, text: &str) -> bool {
        let ran = self.routing.dispatch_change(command, text);
        if ran {
            self.request_frame(FrameReason::Input);
        }
        ran
    }
}
