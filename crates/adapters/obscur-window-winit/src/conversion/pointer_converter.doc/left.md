Builds a [`PointerEventKind::Leave`] event at `position` for pointer `id`, with
the `held` buttons it left under (spec §4.7).

winit's `CursorLeft` carries no position either, and unlike an entry this one
needs none to be correct: the runtime treats a leave as "nothing is under the
pointer" whatever the hit test says, so the last known position is a truthful
record of where it went out rather than a guess about where it is.
