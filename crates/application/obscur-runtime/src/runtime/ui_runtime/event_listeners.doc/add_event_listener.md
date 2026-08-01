Registers a §2.3 **listener** for `command` on `node`, running in `phase`
(spec §2.3).

This is the other half of §2.3, the one no API reached before: a listener
*observes and steers* a command as it propagates, where
[`on_command_at`](Self::on_command_at) registers the single handler that
**claims** it on §2.6's chain. Concretely, a listener can

- see a command aimed at a **descendant** — with
  [`ListenerPhase::Capture`] it sees it *before* the target does, which is
  how an overlay or a gesture arbiter takes an interaction away;
- read [`UiEvent::target`] to learn which descendant was hit while
  [`UiEvent::current_target`] says where the listener itself sits;
- call [`UiEvent::stop_propagation`] to keep the command from travelling
  further, or [`UiEvent::prevent_default`] to let it travel while
  suppressing the handler that would have run;
- call [`UiEvent::stop_immediate_propagation`] to skip the *remaining
  listeners on this same node* — which is why several listeners per node
  are allowed at all.

Unlike a handler, a listener never replaces an earlier one: each call adds
one, and they run in registration order. Registering does not schedule a
frame; a dispatch that actually runs one does.

Listeners on a removed node are unreachable but not reaped automatically,
for the same reason per-node handlers are not — see
[`clear_node_commands`](Self::clear_node_commands), which clears both.

[`UiEvent::target`]: obscur_interaction::event::UiEvent::target
[`UiEvent::current_target`]: obscur_interaction::event::UiEvent::current_target
[`UiEvent::stop_propagation`]: obscur_interaction::event::UiEvent::stop_propagation
[`UiEvent::prevent_default`]: obscur_interaction::event::UiEvent::prevent_default
[`UiEvent::stop_immediate_propagation`]: obscur_interaction::event::UiEvent::stop_immediate_propagation
