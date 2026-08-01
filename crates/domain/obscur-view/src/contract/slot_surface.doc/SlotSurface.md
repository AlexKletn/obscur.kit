The string-keyed *tooling* view of one named slot a component declares (spec
§2.12).

[`SlotContract`](crate::contract::SlotContract) is the **runtime** view: it
names the slot with an [`InternedSymbol`](obscur_identity::InternedSymbol)
and its members with [`ComponentId`](obscur_identity::ComponentId)s, neither
of which resolves back to a string. A `.okui` source writes `slot="header"`,
so the tool that checks it needs the name as written — the same split
[`ComponentSurface`](crate::contract::ComponentSurface) makes against the
typed [`ComponentContract`](crate::component::ComponentContract), and for the
same reason.

A slot declares the three things §2.12 names:

- `name` — the value a child's `slot="…"` must match;
- `cardinality` — how many children may carry that name (see
  [`SlotCardinality`]);
- `children` — which child tags the slot admits (see [`ChildrenSurface`]; only
  `Only` restricts a **slot's** membership — `Any` and `None` both leave it
  unrestricted, because a slot that admits nothing is a slot with nothing to
  assign to it, which its `cardinality` states better. A *component's* `None`
  does now cross into the `.okui` compiler as a real leaf; a slot's does not).

```
use obscur_view::contract::{ChildrenSurface, SlotCardinality, SlotSurface};
let slot = SlotSurface::new("header", SlotCardinality::ExactlyOne).allowing_child("Title");
assert_eq!(slot.name(), "header");
assert_eq!(slot.cardinality(), SlotCardinality::ExactlyOne);
assert_eq!(slot.children(), &ChildrenSurface::Only(vec!["Title"]));
```
