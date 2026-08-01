The embeddable runtime that owns the retained tree, scene, scheduler, and
live interaction (hover/focus) state, and runs the real pipeline when a
frame is due. Pointer input is dispatched into it via `handle_pointer`,
which resolves a click on a button to its bound command and runs the
registered handler; the reactive builder then rebuilds the view.
