The active theme index, shared behind an `Rc<RefCell<_>>` by the per-tab
command handlers (which set it on a click) and the reactive view builder
(which reads it back to rebuild the highlighted view) — the state bridge.
