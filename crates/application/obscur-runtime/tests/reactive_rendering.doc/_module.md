The held view builder: the loop a real host actually runs
(click → app state → rebuild → reconcile), and the explicit path it must not
disturb.

`run_due_frame_reactive` is the entry point a windowed host calls once per
frame with no view in hand: the runtime asks the builder it was given, and
reconciles whatever comes back. Three of these five tests close that loop
end to end — a click dispatches a command, the command mutates an `Rc<Cell<…>>`
the builder reads, and the *rebuilt* tree carries the new number. Reading the
number back out of the retained tree, rather than out of the cell, is the whole
point: the cell only proves the handler ran.

The other two are the negative space, and they are what makes the loop safe to
install. With no builder the reactive path returns `None` and, crucially, leaves
the pending frame request **undrained** — a host that installs a builder late
must not silently lose the frames requested before it. With a builder but
nothing due it returns `None` and renders nothing at all. And the last test
holds the two paths apart: an installed builder is invisible to `render_now` and
to the scheduler-gated `&View` path, both of which render exactly what they were
handed.
