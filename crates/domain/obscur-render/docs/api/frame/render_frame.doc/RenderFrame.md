A [`SceneSnapshot`] paired with the viewport it is to be rendered at
(spec §5.2).

The frame carries the backend-agnostic [`SceneSnapshot`] — the
determinism-boundary artifact of spec §5.2. Its display list fixes the
geometry, paint order, and clipping that every backend must reproduce
identically; only rasterization may differ. The [`Self::viewport`] is the
target area in physical pixels the snapshot renders into.

This type is not `Copy`: it owns a [`SceneSnapshot`], which holds a display
list.
