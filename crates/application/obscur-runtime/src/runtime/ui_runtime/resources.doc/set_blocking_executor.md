Installs the blocking-work facility resource loads run on (spec §3.3).

Explicit and capability-dependent by design: without it the runtime is
still fully functional — a load simply runs on the frame thread, deferred
to after the frame phase (on poll) rather than inside `render_now`.
Installing one moves the load off the frame thread entirely, and the core
stays tied to no particular async runtime, because all it ever sees is this
port (§3.3 "the core is not tied to Tokio").

A completed load returns as a typed message the host drains with
[`poll_resource_tasks`](Self::poll_resource_tasks). A headless test installs
a deterministic executor and gets controlled completion for free (§7.4).
