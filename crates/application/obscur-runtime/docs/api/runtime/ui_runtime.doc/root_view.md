The host-installed view **builder** — the keystone of the reactive
re-render loop (spec §3.1). When set (via
[`set_root_view`](Self::set_root_view)), the reactive frame path
[`run_due_frame_reactive`](Self::run_due_frame_reactive) calls it each due
frame to *rebuild* the desired [`View`] from whatever application state a
command handler just mutated, closing the loop
click → handler mutates state → `request_frame` → rebuild → reconcile →
paint. It is a `FnMut() -> View` that typically captures the same
`Rc<RefCell<_>>`/`Rc<Cell<_>>` app state the command handlers do, so the
view it produces reflects the latest state. `None` until installed, in
which case the reactive path is a no-op — the explicit
[`run_frame`](Self::run_frame)/[`run_due_frame`](Self::run_due_frame)
callers that pass a `&View` directly are entirely unaffected.
