#![doc = include_str!("event_listeners.doc/_module.md")]

use obscur_identity::{InternedSymbol, NodeId};
use obscur_interaction::event::ListenerPhase;

use crate::runtime::IntoCommandHandler;

use super::UiRuntime;

impl UiRuntime {
    #[doc = include_str!("event_listeners.doc/add_event_listener.md")]
    pub fn add_event_listener<M>(
        &mut self,
        node: NodeId,
        command: InternedSymbol,
        phase: ListenerPhase,
        listener: impl IntoCommandHandler<M>,
    ) {
        self.routing.add_listener(node, command, phase, listener);
    }

    #[doc = include_str!("event_listeners.doc/event_listener_count.md")]
    #[must_use]
    pub fn event_listener_count(&self, node: NodeId, command: InternedSymbol) -> usize {
        self.routing.listener_count(node, command)
    }

    #[doc = include_str!("event_listeners.doc/on_text_input.md")]
    pub fn on_text_input(&mut self, command: InternedSymbol, handler: impl FnMut(&str) + 'static) {
        self.routing.set_text_input(command, handler);
    }

    #[doc = include_str!("event_listeners.doc/on_value_input.md")]
    pub fn on_value_input(
        &mut self,
        command: InternedSymbol,
        handler: impl FnMut(f32, f32) + 'static,
    ) {
        self.routing.set_value_input(command, handler);
    }

    #[doc = include_str!("event_listeners.doc/has_text_input_handler.md")]
    #[must_use]
    pub fn has_text_input_handler(&self, command: InternedSymbol) -> bool {
        self.routing.has_text_input(command)
    }

    #[doc = include_str!("event_listeners.doc/on_change.md")]
    pub fn on_change(&mut self, command: InternedSymbol, handler: impl FnMut(&str) + 'static) {
        self.routing.set_change(command, handler);
    }

    #[doc = include_str!("event_listeners.doc/has_change_handler.md")]
    #[must_use]
    pub fn has_change_handler(&self, command: InternedSymbol) -> bool {
        self.routing.has_change(command)
    }
}
