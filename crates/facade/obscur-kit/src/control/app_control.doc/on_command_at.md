Registers `handler` to run when `command` reaches the node `target` names —
links 1 and 3 of §2.6's routing chain.

This is what makes routing **positional**: the same command symbol runs
different logic depending on which node the command originated at. A handler
here claims the command and stops the chain, so no ancestor, no window
handler and no application fallback sees it.

`target` is a §7.5 stable identifier — `test-id="save"` written on the markup
itself, an `automation-id`, or a [`NodeId`] read back from the tree — because
a node id does not exist before the first frame and an application configures
itself before `run`. The name is resolved after every commit and re-resolved
if its node is replaced, so the registration follows the element.

`handler` may take nothing or take the propagating
[`UiEvent`](crate::interaction::event::UiEvent), exactly as
[`UiRuntime::on_command_at`] accepts both.

Registering the same `(target, command)` twice queues two registrations; the
second replaces the first at the runtime, because a per-node *handler* is
singular (a *listener* is not — see
[`add_event_listener`](Self::add_event_listener)).
