The usage contract of a component: what an instance of it must and may carry
(spec §2.13).

A contract names the `component` it constrains and records:

- `required_props` — attribute names every instance must set;
- `allowed_props` — additional attribute names an instance *may* set. The
  prop surface is **closed** once the contract declares *any* prop — required
  or allowed: an attribute is then permitted only if it is in `allowed_props`
  or `required_props`, is a [reserved](is_reserved_attribute) structural
  attribute, or is an `on:*` event (events are governed by `allowed_events`);
  every other attribute is an unknown-attribute violation. A contract that
  declares **no** props at all leaves the surface *open* (any attribute is
  accepted);
- `allowed_events` — the `on:<event>` names an instance may bind; an
  **empty** list means *no event restriction*. A non-empty list closes the
  event surface: an `on:<event>` binding is permitted only if `<event>` is
  listed;
- `children` — which children an instance may hold, as a [`ChildPolicy`]:
  [`Any`](ChildPolicy::Any), [`None`](ChildPolicy::None) (a leaf — see
  [`no_children`](Self::no_children)) or [`Only`](ChildPolicy::Only) the listed
  tags. This is the one surface that is **not** governed by emptiness: it used
  to be a `Vec` whose empty state meant "any child", which left "no children"
  inexpressible for every component that synthesises its own content;
- `slots` — the named slots the component declares (§2.12). An **empty** list
  closes the slot surface rather than opening it: a `slot="…"` on a child of
  a component that declares none names nothing, and there is no permissive
  reading of that to fall back to.

Contracts are built fluently, for example:

```
use obscur_okui_compiler::validation::ComponentContract;
let contract = ComponentContract::new("Tabs")
    .requiring_prop("selected")
    .allowing_prop("orientation")
    .allowing_event("change")
    .allowing_child("Tab");
assert_eq!(contract.component(), "Tabs");
```
