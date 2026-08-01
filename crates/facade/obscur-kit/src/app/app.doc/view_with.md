Sets the root to a builder that also receives the frame's [`FrameInfo`] —
the viewport and the device scale.

Use this when what to build depends on how much is visible: a long list
should window itself to the rows the viewport can show rather than emit one
node per item. Overrides any previous [`view`](Self::view) /
[`screen`](Self::screen).
