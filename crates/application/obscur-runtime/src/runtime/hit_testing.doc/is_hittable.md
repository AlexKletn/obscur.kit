Whether `node` actually receives the hit at these points, beyond mere
bounding-box containment (§4.7 "clipping", "custom shapes").

* **Clipping** — every logical ancestor whose computed `overflow` is not
  `visible` clips its subtree to its border box, which is a fixed on-screen
  rectangle; the `screen` point must lie inside each such ancestor's box, or
  `node` is clipped away.
* **Shape** — when `node` has a `border-radius`, the `mapped` point must lie
  inside the rounded border box, not merely its rectangular bounds, so a
  corner cut-out falls through to whatever is beneath.
