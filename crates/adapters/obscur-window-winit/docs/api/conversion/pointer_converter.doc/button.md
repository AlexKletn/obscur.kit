Maps a winit [`MouseButton`](winit::event::MouseButton) to the domain
[`PointerButton`], answering `None` for
[`Other`](winit::event::MouseButton::Other).

That `None` is a decision, not a fall-through, and this is where it is recorded.

**The domain models the five buttons the web platform names**, and it models them
by the same numbers: `PointerButtons`' masks are `1`, `2`, `4`, `8`, `16` for
primary, secondary, middle, back and forward — the DOM's `MouseEvent.buttons`
bitmask exactly. `Other(code)` is not a sixth button in that vocabulary; it is a
raw platform code, and the same `code` names different hardware on different
sessions. There is nothing to map it *to*.

**Nor is it folded onto a modelled neighbour.** Reporting a gaming mouse's thumb
button as a `Middle` would be a press the user never made, delivered to a handler
with no way to tell it from a real one — worse than a button nobody hears,
because the application can defend against neither. The set is not full (three of
`PointerButtons`' eight bits are spare), so this is a choice about meaning rather
than about room: a bit meaning "some button, we do not know which" is a bit
nothing can act on.

**The drop is symmetric, which is what keeps the held set honest.** The caller —
`obscur-kit`'s `app/runner.rs`, on `WindowEvent::MouseInput` — drops the whole
event when this answers `None`, so an unmodelled button contributes neither a
press nor a release. `held_after` never sees it in either direction, and the
chord the runtime believes is held stays exactly the presses that were delivered.
Mapping the press while dropping the release is the failure this symmetry avoids:
that one leaves a button held forever.
