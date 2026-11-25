Registers a §2.3 **listener** for `command` on the node `target` names, running
in `phase`.

A listener observes and steers a command as it propagates, where a handler
claims it. With [`ListenerPhase::Capture`] it sees a command aimed at a
*descendant* before the target does — an overlay or a gesture arbiter taking
the interaction away — and it can call
[`stop_propagation`](crate::interaction::event::UiEvent::stop_propagation) to
end the propagation or
[`prevent_default`](crate::interaction::event::UiEvent::prevent_default) to let
it continue with the command's handler suppressed. Several listeners per node
are allowed and run in registration order.

Same targeting rules as [`on_command_at`](Self::on_command_at).
