Whether `node` actually receives the hit at `screen`, beyond mere
bounding-box containment (§4.7 "clipping", "custom shapes", "pointer-event
policy").

* **Pointer-event policy** — a node whose computed `pointer-events` is
  `none` is transparent: the pointer passes through to whatever is behind
  it. Because the property inherits, its descendants have computed `none`
  too and are transparent by the same clause — while a descendant that
  declared `pointer-events: auto` computed `auto` and stays hittable.
* **Clipping** — every logical ancestor whose computed `overflow` is not
  `visible` clips its subtree to its border box. Each ancestor is tested at
  ITS OWN point: `screen` scrolled back by the containers above *it*, which
  excludes its own scroll, since a container's box does not move when its
  content does. Testing every ancestor at the raw screen point — as this
  did before — is right only for the first clipping ancestor: a scroller
  nested inside another scroller states its box in the outer one's
  unscrolled space, so the point has to be folded that far before the
  comparison means anything.
* **Shape** — when `node` has a `border-radius`, `node`'s own folded point
  must lie inside the rounded border box, not merely its rectangular
  bounds, so a corner cut-out falls through to whatever is beneath.

Both geometric tests read their rect from [`LayoutBoxes`], which is
*pre*-transform, so both take their point through
[`local_point`](Self::local_point) first. For the untransformed node that
is the identity and the test is unchanged.

The clip is the ancestor's BORDER box, where CSS clips overflow to the
padding box. That is deliberate agreement rather than an oversight: the
compositor clips to `layout_box.bounds` and `scroll_containers` takes the
same rect as the scrollport, so hit-testing to the padding box would make
the pointer disagree with the pixels. Narrowing it is one change across all
three, recorded in `docs/api/12-reality.md`.
