Stacking (paint) order for positioned elements: a bounded, flat CSS
`z-index` model.

Full CSS 2.1 stacking contexts (Appendix E) are intricate — nested contexts,
the `auto`/non-`auto` split, opacity/transform-induced contexts, and the
seven-layer within-context ordering. This module implements a **flat**,
tractable subset that is correct for the common case of a handful of
positioned elements with `z-index` overlapping:

* Every node is assigned an **effective stacking z** (an `i32`). A node's z is
  its own `z-index` when it is **positioned** (`position != static`) and its
  `z-index` is a numeric `<integer>`; otherwise it inherits its parent's
  effective z (root = `0`). `z-index: auto` therefore stacks *with* the parent
  (effective z of the nearest positioned-with-integer ancestor), so a
  positioned subtree with a `z-index` moves together in z — its background and
  its descendants share one stacking level.
* The scene collects each node's own paint (its background / text — not its
  descendants') as a *paint unit* keyed `(z, tree_index)` and STABLE-SORTS the
  units by `(z ascending, tree_index ascending)`. Higher z paints last (on
  top); ties keep document (pre-order) order.

On top of the effective z, each node also carries a **§9.9 paint-layer rank**
(see [`PaintLayer`] / [`effective_paint_layer`]) that orders units *within an
equal-z bucket*. CSS 2.1 §9.9.1 / Appendix E paint a stacking context's
contents in seven layers, bottom→top: (1) the context root's background &
borders; (2) child contexts with negative z; (3) non-positioned in-flow
**block** descendants; (4) non-positioned **floats**; (5) non-positioned
**inline** descendants; (6) **positioned** descendants with `z-index: auto`
**or** `0`; (7) child contexts with positive z. The critical consequence — and
the one the old flat `(z, tree_index)` sort got wrong (catalog bug B2) — is
that **layer 6 always paints above layers 1–5 regardless of tree order**: a
positioned `z:auto`/`0` box lifts above its *non-positioned* siblings even when
it appears earlier in the document. Negative- and positive-z positioned boxes
are already separated by the primary `z` key (layers 2 and 7), so the rank only
has to lift positioned-`z0`/`auto` boxes (layer 6) above non-positioned content
(layers 1–5) in the common `z == 0` bucket.

**Not modelled** (documented simplifications): nested stacking-context
isolation (a child's high z can escape its parent's context, unlike real
CSS) — the [`effective_paint_layer`] walk *inherits* the lift so a positioned
subtree moves together, but it does not re-isolate a descending context; the
`auto`-establishes-no-context nuance; opacity/transform/filter induced stacking
contexts; and — because `float` is not a computed property here and the
byte-identical regression guard forbids reordering non-positioned block vs
inline siblings — the §9.9 non-positioned layers 1/3/4/5 are **collapsed into a
single rank** ([`PaintLayer::NonPositioned`]). When no node authors a
non-`auto` `z-index` *and* no node is positioned, every effective z is `0` and
every layer rank is [`PaintLayer::NonPositioned`], so the stable sort by
`tree_index` reproduces the exact pre-order paint sequence — the regression
guard.
