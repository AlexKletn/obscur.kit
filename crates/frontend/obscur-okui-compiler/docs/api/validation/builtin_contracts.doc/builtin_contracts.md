Returns the contracts for every built-in view primitive (spec §2.6), keyed
later by their tag name in the [`ContractRegistry`](crate::compile::ContractRegistry).

The set mirrors the `obscur-primitives` builder surfaces:

- `Container` — a grouping box: no attributes of its own, any children.
- `Text` — a literal text run: no attribute surface (its content *is* its
  child text), any children.
- `Image` — requires `src`, optionally `alt`; a leaf.
- `Scroll` — optional `horizontal`/`vertical` axis flags, any children.
- `Canvas` — requires `width` and `height`; a leaf.
- `Portal` — requires `target`, any children.
- `Layer` — optional `opacity`, any children.
- `ForeignSurface` — requires `surface`; a leaf.

A primitive that declares no attributes (`Container`, `Text`) leaves its
prop surface *open* — the string contract model treats "no declared props"
as unrestricted — so those tags stay permissive; every primitive that
declares props closes its surface and rejects unknown attributes.

# The three leaves say so now

"A leaf" was a claim in this comment and nowhere in the data: an empty
allowed-children list meant *any* child, so `<Image src="a.png">caption</Image>`
compiled and authored children into a node that paints one bitmap and never
looks at them. `Image`, `Canvas` and `ForeignSurface` therefore declare
[`no_children`](ComponentContract::no_children) — the [`ChildPolicy::None`]
state added for §2.13's "allowed children" — and the comment and the contract
now agree.

This is a claim about the **component tag**, not about every spelling that
binds to the same widget: contracts are keyed by the authored tag, so a
lowercase `<img>`/`<canvas>` element (which `name_binding::builtin_component_id`
resolves to the same primitive) has no contract registered and is untouched.
See [`widget_contracts`]'s note on `<button>` for why that distinction is load
bearing rather than an oversight.

[`ChildPolicy::None`]: crate::validation::ChildPolicy::None
