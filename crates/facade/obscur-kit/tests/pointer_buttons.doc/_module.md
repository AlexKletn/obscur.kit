The buttons the model always had and the platform layer always threw away,
reached the way an application reaches them.

`PointerButtons` has modelled `PRIMARY`, `SECONDARY`, `MIDDLE`, `BACK` and
`FORWARD` since §2.4 was first implemented, and `PointerConverter::button`
mapped all five from winit — with no caller. The runner's `MouseInput` arm was
guarded to `MouseButton::Left` and its event builder hardcoded `PRIMARY`, so the
other four never reached the runtime at all. There was no context menu because
there was no right click.

Three separate claims are pinned here, because they fail independently:

- **The set is a set.** winit reports one transition per event and never the
  set, so a chord — two buttons held at once — exists only if something folds
  the transitions. The fold and its unit cases live with the runner; what this
  file pins is the consequence: a release of one button does not consume the
  press of another.
- **Only the primary button engages.** A right or middle press moves focus and
  stops: no `:active`, no caret, no armed pan, and no activation on release. A
  right click that ran a button's `on:click` would be the loudest possible way
  to discover the guard had been lifted.
- **`contextmenu` has an emitter.** It is dispatched from a secondary press and
  from nothing else, and refusing it suppresses the focus move — which is the
  whole of what a secondary press does, so the refusal is stated in full rather
  than gestured at.

The crossing half is here too: `PointerEventKind::Leave` now empties the hover
chain instead of asking the hit test what is under a pointer that has gone. That
is what makes `WindowEvent::CursorLeft` worth converting — without it a leave
would keep the row it exited by hovered, which is exactly the state the unhandled
event left behind.

Dispatch inside the winit event loop is owner-verified; nothing here claims to
have run it. What these cases exercise is `UiRuntime`'s public door, with events
shaped the way the converter now shapes them.

Not a `spec_*` file: the spec-derived suites are an oracle written without
reading this repository.
