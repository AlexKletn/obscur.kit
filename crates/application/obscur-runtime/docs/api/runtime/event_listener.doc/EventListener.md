A single listener registered on a node by
[`UiRuntime::add_event_listener`](super::UiRuntime::add_event_listener)
(spec §2.3).

A listener is *not* the same thing as a §2.6 command **handler**, and the
difference is worth stating because both are closures on a node:

- a **handler** ([`on_command_at`](super::UiRuntime::on_command_at)) is the one
  piece of logic that *claims* a command; §2.6's chain offers the command to
  each link in turn and stops at the first that has one. There is at most one
  per `(node, command)`, and registering again replaces it.
- a **listener** *observes and steers* the same command as it propagates. Any
  number may sit on one node, they run in registration order, and none of them
  claims anything — they are §2.3's capture and bubble phases, which exist
  precisely so a node that is not the target gets a say.

That is why listeners are an ordered `Vec` while handlers are a single slot:
`stop_immediate_propagation` — *"skip the remaining handlers on the current
node"* — has nothing to skip unless a node can carry more than one, so the
spec's own API implies the multiplicity.
