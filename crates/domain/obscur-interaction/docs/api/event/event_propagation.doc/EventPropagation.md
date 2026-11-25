The propagation-control flags an event carries while it travels its path
(spec §4.7).

Mirrors the DOM control surface a handler uses to steer routing:
[`stop_propagation`](Self::stop_propagation) halts travel after the current
node finishes, [`stop_immediate_propagation`](Self::stop_immediate_propagation)
halts it at once (skipping any remaining handlers on the current node), and
[`prevent_default`](Self::prevent_default) suppresses the host's built-in
action. This crate stores the flags; honoring them is the runtime's job.
