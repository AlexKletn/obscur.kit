§2.6 link 4: handlers bound to this window surface, keyed by command —
application logic to run when a dispatched command reaches this window
unclaimed by anything positional (spec §2.3, §3.1 — events flow up to a
command, the command runs a handler).

A `UiRuntime` *is* one window surface — it owns one tree, one viewport and
one focus — so a handler registered on the runtime itself is a
window-level handler by construction. The *application* fallback, link 5,
spans windows and therefore lives in the shared
[`app_commands`](Self::app_commands) handle instead.

Exactly one handler per command (a later registration replaces an earlier
one), so this link is a deterministic single lookup — no iteration order
observed.
