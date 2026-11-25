Arranges the tree and returns whether any box was produced.

[`LayoutPhase::Skip`] returns `false` without touching anything: the
retained boxes and hit-test index carry over unchanged, which is the whole
point of the classification.

`editing` threads the runtime-owned editing session into the measure pass so
the focused editable's text is SIZED from the live session buffer (the SAME
string, RAW, that compose paints), not the retained tree value that lags by
a keystroke — so the arranged box height (line count) equals the painted
line count on a `white-space: normal`/`pre-wrap` field (CSS Text 3 §4.1.1).
[`None`] when no session is active ⇒ byte-identical to the retained-tree
measure.

`viewport_scroll` is the surface's scroll offset (test/host-driven;
`(0, 0)` by default), which engages `position: sticky` clamping this frame.

`inputs` are the non-style inputs these boxes are arranged under; they are
recorded only on a frame that actually arranges.
