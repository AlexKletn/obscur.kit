Registers a §2.3 **listener** for `command` on the node `target` names,
running in `phase`.

A listener observes and steers a command as it propagates, where
[`on_command_at`](Self::on_command_at) registers the one handler that claims
it. With [`ListenerPhase::Capture`] it sees a command aimed at a *descendant*
before the target does — which is how an overlay or a gesture arbiter takes
an interaction away — and it may call
[`stop_propagation`](crate::interaction::event::UiEvent::stop_propagation) or
[`prevent_default`](crate::interaction::event::UiEvent::prevent_default) to
steer what happens next.

Several listeners per node are allowed and run in registration order.
