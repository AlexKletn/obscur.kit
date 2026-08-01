Composes this frame's display list and installs it as the scene root.

`editing` threads the runtime-owned editing session into compose so the
focused editable paints its live session text plus a caret + selection
highlight (spec §5.11) — the SAME [`EditingView`] the measure pass sized the
box with, so paint and box agree (CSS Text 3 §4.1.1). With no session the
view is [`None`] and compose is byte-identical.

`hovered` is the topmost node under the pointer, which the §7.6 inspect
overlay outlines when inspect mode is on.
