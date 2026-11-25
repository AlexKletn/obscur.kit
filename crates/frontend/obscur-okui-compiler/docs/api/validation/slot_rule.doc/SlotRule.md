One named slot a component declares, in the string (tooling) contract model
the `.okui` compiler and the `okui!` macro both consult (spec §2.12).

A `.okui` source assigns a child to a slot by name — `<Title slot="header"/>`
— so the rule that checks the assignment must hold the name as written. The
symbol-keyed [`SlotContract`](obscur_view::contract::SlotContract) is the
runtime counterpart; see
[`SlotSurface`](obscur_view::contract::SlotSurface) for the same split at the
component-surface level.

[`cardinality`](Self::cardinality) is
[`SlotCardinality`](obscur_view::contract::SlotCardinality) itself, not a
re-spelling of it: the string and typed models disagree about *names*, which
is the whole reason both exist, but they must not disagree about how many
children a slot takes.

`allowed_children` is empty when the slot admits any child. A **component**'s
child surface is no longer a list like this one — it is a three-state
[`ChildPolicy`](crate::validation::ChildPolicy), because a component had to be
able to say it takes *no* children (§2.13) and an empty list already meant
"any". A slot needs no third state: a slot admitting nothing is a slot nothing
may be assigned to, which its [`cardinality`](Self::cardinality) states
directly.
