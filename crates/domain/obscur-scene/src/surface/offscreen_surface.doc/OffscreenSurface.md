A descriptor for an offscreen render target (Obscur Kit 2 specs §5.6).

Offscreen output renders a subtree or scene to a texture, bitmap, thumbnail,
preview, or node screenshot. This is a pure data descriptor — Phase 1
captures the target `size` (in logical pixels) and the `scale` (DPI) to
render at. Additional inputs described by §5.6 — theme, locale, and
deterministic-resource configuration — are added in a later phase.

The physical pixel size of the target is [`Self::physical_size`], each axis
of `size` multiplied by `scale`.
