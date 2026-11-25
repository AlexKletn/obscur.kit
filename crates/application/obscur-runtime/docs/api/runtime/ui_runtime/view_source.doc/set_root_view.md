Installs the runtime-held **view builder** that closes the reactive
re-render loop (spec §3.1), replacing any previously installed builder.

Once installed, [`run_due_frame_reactive`](Self::run_due_frame_reactive)
calls `builder` each due frame to *rebuild* the desired [`View`] from
current application state — so a command handler that mutates app state (via
a captured `Rc<RefCell<_>>`/`Rc<Cell<_>>`) and requests a frame produces a
freshly-built, reconciled view on the next due frame with **no** view passed
in by the caller. `builder` typically captures the *same* state the
[`on_command`](Self::on_command) handlers mutate.

This is additive: the explicit
[`run_frame`](Self::run_frame)/[`run_due_frame`](Self::run_due_frame)/[`render_now`](Self::render_now)
paths that take a `&View` are unaffected, and with no builder installed the
reactive path is a no-op.
