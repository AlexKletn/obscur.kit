Virtual collections: the reserved marker a `<VirtualFor>` lowers to, and the
viewport-bounded realization that replaces it with a window of rows (spec
§2.11, §4.9).

# What §4.9 asks for

§2.11 names two collection constructs, `<For>` and `<VirtualFor>`, and they
are not two spellings of one thing. A `<VirtualFor>` is a *performance
contract*: §4.9 says it materializes a **window** of the collection rather
than the whole of it, widened by overscan, with the geometry of what is not
yet measured *estimated*. A `<For>` over a million rows builds a million
nodes; a `<VirtualFor>` over the same collection builds only the handful the
viewport can show.

# Why a marker, and why realization runs in the runtime

The window a `<VirtualFor>` realizes is a function of the **viewport** — how
tall it is, how far it has scrolled — and of an **estimated row extent**. A
*view builder* runs before layout and is handed neither, so the front end
cannot expand the window itself: it lowers `<VirtualFor>` to a single
[`VIRTUAL_COLLECTION`] marker node carrying the per-row template as its
children and the estimated row extent as a reserved property.

The runtime, which owns the viewport, then substitutes each marker away —
exactly as [`project_slots`](super::project_slots) substitutes a slot outlet
— replacing it, *before the tree is reconciled*, with as many clones of its
template as the window holds. The window count comes from
[`obscur_layout::virtualization::VirtualLayout`], so it is bounded by the
viewport and independent of the collection's length. That count-independence
is the whole point of the construct, and is what a `<For>`, which splices its
template once and lets the runtime materialize one node per item, does not
have.

# Why a reserved `ComponentId` and not a `ViewNode` variant

Because a marker never becomes a retained node — realization substitutes it
away before reconcile, the same reason [`SLOT_OUTLET`](super::SLOT_OUTLET) is
a reserved id rather than a third enum arm. A `ViewNode` variant would add a
case to every exhaustive match over the enum to describe a node no consumer
of the retained tree can ever see.
