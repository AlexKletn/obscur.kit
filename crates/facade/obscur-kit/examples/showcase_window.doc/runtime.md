The embeddable runtime that owns the retained tree, scene, scheduler, and
live interaction (hover/focus/editing) state, and runs the real pipeline
when a frame is due. The shared [`ShowcaseState`] lives behind an
`Rc<RefCell<_>>` owned jointly by the command / text-input handlers (which
mutate it) and the reactive view builder (which reads it back to rebuild
[`showcase::view`] each frame) — both installed on this runtime, so the
state needs no separate field here.
