Morphs a shared element from its box in an OLD view to its box in a NEW view
(spec §4.17 "shared-element transitions").

Identified by a stable [`SharedElementKey`], the element preserves its
identity across the view change while its layout box animates from
[`from_bounds`](Self::from_bounds) (its position/size in the old view) to
[`to_bounds`](Self::to_bounds) (its position/size in the new view) — the same
FLIP interpolation as [`LayoutTransition`](super::LayoutTransition). The
runtime pairs elements by key across views and supplies the from/to bounds;
this type holds them and interpolates.
