The string-keyed *tooling* view of a component's usage contract (spec §2.13).

The typed [`ComponentContract`](crate::component::ComponentContract) is the
**runtime** view: it names every prop/event/child by an interned
[`InternedSymbol`](obscur_identity::InternedSymbol) (a hash of the string)
with no reverse string resolution, so it cannot drive a string-keyed tool
such as the `.okui` compiler's contract checker. `ComponentSurface` is the
parallel **tooling** view: it keeps the very same names as plain strings, so
the compiler can match them against the attribute/event/child names written
in a `.okui` source and enforce §2.13 for user-defined components.

`#[derive(Contract)]` emits both — the typed `contract()` and this
`contract_surface()` — from the same string literals, before the typed one
hashes its names away.

Its fields mirror the compiler's string contract surface exactly:

- `required_props` — attribute names every instance must set;
- `allowed_props` — additional attribute names an instance *may* set (the
  prop surface is closed once any prop, required or allowed, is declared);
- `allowed_events` — the `on:<event>` names an instance may bind (an empty
  list leaves the event surface open);
- `children` — which child tags are permitted (see [`ChildrenSurface`]);
- `slots` — the named slots the component declares (see [`SlotSurface`]).
  Declaring none closes the slot surface: a child of an instance may then
  carry no `slot="…"` at all.

It is built fluently, for example:

```
use obscur_view::contract::{ChildrenSurface, ComponentSurface};
let surface = ComponentSurface::new("Tabs")
    .requiring_prop("selected")
    .allowing_prop("orientation")
    .allowing_event("change")
    .allowing_child("Tab");
assert_eq!(surface.component(), "Tabs");
assert_eq!(surface.required_props(), &["selected"]);
assert_eq!(surface.children(), &ChildrenSurface::Only(vec!["Tab"]));
```
