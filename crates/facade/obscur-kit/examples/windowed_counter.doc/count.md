The counter value, shared between the click handlers (which mutate it) and
the reactive view builder (which reads it) — the canonical `Rc<Cell<i32>>`
bridge. Registered handlers for `CMD_INCREMENT`/`CMD_DECREMENT` bump it,
and the [`UiRuntime::set_root_view`] builder rebuilds the counter view from
its current value each reactive frame.
