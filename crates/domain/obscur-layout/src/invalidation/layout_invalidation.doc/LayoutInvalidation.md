Accumulates the set of layout contexts that must be recomputed, each tagged
with the typed [`LayoutInvalidationReason`] that first invalidated it
(spec §4.3).

# Status: the layout phase consumes this, on both halves

`UiRuntime::layout_phase` populates one of these from real per-node style
deltas and hands it to `LayoutCoordinator::layout_invalidated`, which
classifies it once into the two sets its two caches need. **Measure**: it drops
exactly the `MeasureCache` entries this set can have invalidated and serves
every other node's size without re-measuring. **Arrange**: it drops the
`ArrangeCache` keys of the upward closure of that set, and a node whose key
survives *and* whose recomputed origin/constraints still match is served from
its retained absolute boxes instead of re-running its subtree's algorithms. So
a hover that changes one node's width re-measures and re-arranges that node and
its ancestor chain, not the tree.

What that does **not** yet cover, stated plainly:

* **This set alone never decides that a node may be skipped, and must not.**
  `arrange` writes *absolute* positions, so a node the set does not mention
  still moves when an earlier sibling grows. The retained-box path is gated on
  this set **and** on the node's freshly recomputed arrange key; §4.3's
  propagation is still performed rather than inferred from this accumulator.
* **The walk is still O(nodes).** A served subtree is still traversed to copy
  its retained boxes into the frame's map. Layout-algorithm runs and text
  shaping are the costs this set removes, not the traversal.
* **The `note_*` gates below are not what the runtime calls.** It records
  through [`invalidate`](Self::invalidate) /
  [`note_style_changed`](Self::note_style_changed) only. The downward gate is
  instead exact-by-construction in the measure cache's key (a child
  re-measures precisely when its constraints differ), and the upward gate is a
  whole-ancestor-chain closure rather than a
  [`PropagationResult`]-driven early stop, because nothing populates a
  [`LayoutContext`]'s [`LayoutDependency`](crate::context::LayoutDependency) in
  production. Until something does,
  [`note_constraints_changed`](Self::note_constraints_changed),
  [`note_child_changed`](Self::note_child_changed) and
  [`note_upward`](Self::note_upward) are offered to a consumer that does not
  exist yet — read them as a contract, not as shipped behaviour.

It is **decoupled from the tree**: it neither stores nor walks parent/child
links. The runtime performs the actual tree walk — finding parents, children,
and their [`LayoutContext`]s — and feeds the observed changes here. This type
only records and merges pending work and, through its `note_*` methods, offers
the §4.3 gate decisions:

- **propagate down only on constraint change** — [`note_constraints_changed`](Self::note_constraints_changed),
- **propagate up only on geometry change** (and stop at containment) —
  [`note_upward`](Self::note_upward),
- re-measure a context when one of its depended-on children changed —
  [`note_child_changed`](Self::note_child_changed).

Merging is deterministic: on a repeat invalidation of the same node the
**first** recorded reason is kept, and no operation ever panics.
