Clamps an item's cross-axis **border** extent into its authored cross-axis
min/max (CSS Flexbox Level 1 §9.4 step 11 "clamp by min/max cross size").

The main axis has always applied its bounds (via [`clamp_floor`]); the cross
axis dropped them entirely, so an item's `max-height` in a row — or a stretch
that should stop at `max-height` — was ignored. Bounds are resolved against
the container cross extent and mapped through the item's `box_sizing` into the
border extent this value is expressed in, matching how the explicit cross size
is resolved in [`cross_item_size`](super::flex_cross_size::cross_item_size). Max
applies before min, so min wins on
conflict (per CSS).

An **intrinsic** cross bound (`max-height: max-content` on a row item) is
still dropped: `resolve` answers [`None`] for a keyword, so it imposes no
limit here. Only the MAIN axis measures its keywords (see
[`intrinsic_main`](super::flex_main_probe::intrinsic_main)).
