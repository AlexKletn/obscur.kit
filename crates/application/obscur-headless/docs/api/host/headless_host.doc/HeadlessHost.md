The official windowless, GPU-less entry point for reproducible tests
(spec §7.4).

A `HeadlessHost` drives a real [`UiRuntime`] — the same integrator a windowed
app runs — with no OS window and no GPU, and adds the two facilities §7.4
names: a [`DeterministicClock`] and a [`DeterministicExecutor`]. Construct
it, [`render_styled`](Self::render_styled) desired view trees, and inspect the
resulting scene and tree.

# What "deterministic" means here

The host owns time and background work, and **a rendered frame goes through
both**:

- [`advance_time`](Self::advance_time) advances the clock and hands the same
  delta to the runtime's motion scheduler, so an animation's sampled value
  reaches the *next* frame's paint. Time never moves on its own: with no
  `advance_time` call, repeated renders are byte-identical.
- The host installs its executor as the runtime's blocking-work port
  ([`UiRuntime::set_blocking_executor`]), so a frame's resource loads are
  *queued* rather than run inline. [`run_tasks`](Self::run_tasks) runs them
  and applies the completions — §7.4's "controlled async completion".

The same sequence of calls with the same inputs always produces the same
results, because every stage is deterministic and neither time nor background
work advances unless the test drives it.

# Example flow

1. `let mut host = HeadlessHost::new(runtime_id);`
2. `let result = host.render_styled(&desired, &sheets);` — one frame.
3. Inspect `result` (phase flags, snapshot) or [`snapshot`](Self::snapshot) /
   [`tree`](Self::tree) for the current committed state.

Anything the host does not wrap is reachable through
[`runtime_mut`](Self::runtime_mut) — motion registration, resource providers,
synthetic input, and the rest of the runtime's surface.
