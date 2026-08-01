Bridges a domain [`ComponentSurface`] into the compiler's string
[`ComponentContract`] (spec §2.13).

`#[derive(Contract)]` emits, alongside the symbol-keyed typed
`obscur_view::component::ComponentContract` (the runtime view), a string
[`ComponentSurface`] (the tooling view) carrying the very same prop/event/
child names as plain strings. This module turns that surface into the
string [`ComponentContract`] the [`ContractValidator`](super::ContractValidator)
consults, so a user component's usage is enforced exactly as a built-in's
is.

The mapping is one-to-one with the string contract's semantics: each
required prop becomes [`requiring_prop`](ComponentContract::requiring_prop),
each allowed prop [`allowing_prop`](ComponentContract::allowing_prop), each
event [`allowing_event`](ComponentContract::allowing_event). Children cross
whole, case for case, into the [`ChildPolicy`] that mirrors this very enum:
[`Only`](ChildrenSurface::Only) to
[`Only`](crate::validation::ChildPolicy::Only),
[`Any`](ChildrenSurface::Any) to [`Any`](crate::validation::ChildPolicy::Any),
and [`None`](ChildrenSurface::None) to
[`no_children`](ComponentContract::no_children).

That last case used to be **flattened into `Any`**: the string contract had a
`Vec` whose empty state already meant "any child", so a derived
`#[contract]` declaring a leaf component crossed this boundary and came out
unrestricted, and `<MyIcon>text</MyIcon>` compiled clean against a contract
that had said the opposite. The declaration was not lost in the derive or on
the domain surface — only here, on the last hop before the checker. With
[`ChildPolicy`] carrying all three states, the bridge is lossless.

Each declared [`SlotSurface`] becomes one [`SlotRule`] (§2.12), carrying its
name, its [`SlotCardinality`](obscur_view::contract::SlotCardinality) — the
very same type, not a re-spelling — and its own child rule. A **slot's**
children are still `Only`-maps-and-nothing-else, because [`SlotRule`] keeps a
tag list of its own: a slot with no declared children is a slot that admits
any, and a slot declaring `None` is a slot nothing may be assigned to, which
the cardinality already expresses better.

[`ChildPolicy`]: crate::validation::ChildPolicy
