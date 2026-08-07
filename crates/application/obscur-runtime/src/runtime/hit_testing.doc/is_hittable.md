Whether `node` actually receives the hit at these points, beyond mere
bounding-box containment (§4.7 "clipping", "custom shapes", "pointer-event
policy").

* **Pointer-event policy** — a node whose computed `pointer-events` is
  `none` is transparent: the pointer passes through to whatever is behind
  it. Because the property inherits, its descendants have computed `none`
  too and are transparent by the same clause — while a descendant that
  declared `pointer-events: auto` computed `auto` and stays hittable.
* **Clipping** — every logical ancestor whose computed `overflow` is not
  `visible` clips its subtree to its border box; the `screen` point,
  brought back into that ancestor's own space, must lie inside its box, or
  `node` is clipped away.
* **Shape** — when `node` has a `border-radius`, the `mapped` point must lie
  inside the rounded border box, not merely its rectangular bounds, so a
  corner cut-out falls through to whatever is beneath.

Both geometric tests read their rect from [`LayoutBoxes`], which is
*pre*-transform, so both take their point through
[`local_point`](Self::local_point) first. For the untransformed node that
is the identity and the test is unchanged.
