Builds a [`PointerEventKind::Enter`] event at `position` for pointer `id`, with
the `held` buttons it arrived under (spec §4.7).

winit's `CursorEntered` carries no position of its own, so a caller must supply
the first one it actually learns — see the runner, which promotes the
`CursorMoved` that follows rather than guessing from the position the pointer
left by.
