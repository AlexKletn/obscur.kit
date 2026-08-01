The frontend name-binding registry: the single policy that maps authored
`.okui`/DSL **names** onto the engine's real handles.

A `.okui` document (or the TSX-like DSL) names things with strings — a tag
like `Button` or `div`, an attribute like `class` or `src`, an event like
`click`. Both frontend lowering paths must agree on what those names bind to:

- the **runtime bridge** ([`crate::document`] → `obscur-view` `View`), which
  interprets the IR into a desired tree at runtime, and
- the **code generator**, which emits ordinary Rust that builds the same
  `View` at *compile* time.

Centralising the policy here keeps the two paths byte-for-byte consistent and
removes the duplicated ad-hoc hashing they used to carry.

# Mapping

- **Tags → [`ComponentId`]**: the built-in widgets and non-container
  primitives map to their stable `COMPONENT_ID`. What an unrecognised tag maps
  to depends on the [`TagPosition`] it was written in — an *element* tag
  (`div`, `span`, `header`) maps to the generic [`Container`](CONTAINER)
  primitive, so it is a real, styleable box; a *component* tag (`Card`) maps
  to [`user_component_id`], the id `#[derive(Contract)]` computes for the very
  same name.
- **Tags → a local name (CSS *type*)**: an *element* tag is also the node's
  own **local name**, the string a §4.13 type selector matches — `<button>`
  is what `button { … }` styles, exactly as in HTML. [`local_name_for`] is
  that policy, and it is the reason both lowering paths emit a [`TYPE`]
  property nobody authored.
- **Attribute names → [`InternedSymbol`]**: the CSS/reserved attributes
  (`class`, `id`, `type`, `src`, `disabled`, `checked`) map to the reserved
  `obscur-view` symbols the reconciler and matcher read; any other name is
  deterministically hashed into a high symbol namespace (top bit set) that
  never collides with the reserved low range. Exactly one binding depends on
  the tag the attribute sits on — `name` on a [`OUTLET_TAG`] — which is why
  [`attr_symbol_on`] exists alongside [`attr_symbol`].
- **Event names → [`InternedSymbol`]**: `click`/`input`/`change` map to the
  reserved `EVENT_*` symbols the runtime's dispatcher resolves; any other name
  is hashed like an attribute.

# Why a component tag is not a box

It used to be. [`component_id_for`] took a tag and nothing else, and resolved
**every** unrecognised name to [`CONTAINER`] — so `<Card/>` was not a
mis-checked component and not a rejected one, it was a generic div, silently.
§2.13's contract validation "skips any tag with no registered contract" was
checking nothing, because the tag was never going to *be* a component.

The position is now an argument rather than a guess, and it is the
distinction both front ends already make and then threw away (the `.okui`
lowering's `starts_uppercase`, the macro's `is_component`, both of which
produce a `UiIrNodeKind::Component`). A caller must now *state* which
fallback it wants, so the wrong one can no longer be picked by omission —
though it remains perfectly spellable, and only the call sites' agreement
with their node's kind keeps it right.

# Why an element tag is also a name

It was not. A tag resolved to a [`ComponentId`] and to nothing else, and a
`ComponentId` is not a selector input — no part of matching reads one. So
every authored element reached the tree **anonymous**: `<button>`, `<div>` and
`<td>` were three nodes with no local name, and `button { … }`, `div { … }`,
`td { … }` — including the rules the facade's own UA stylesheet is written
entirely in — matched none of them. Only the hand-written widgets styled
themselves, by setting the reserved [`TYPE`] property in Rust
(`obscur_widgets::Button::view`). Authored markup could be styled by class and
by id, and by type never.

[`local_name_for`] is the missing half: an element tag is the node's local
name as well as its id, so a `.okui`/`okui!` `<button>` is a real base element
in the HTML sense — the UA sheet's `button { … }` reaches it, and so does an
author's. Both lowering paths call it, and neither may open-code the rule: the
bytes they emit have to stay identical, which is what this module is for.

# The id of a component nobody here can see

§2.14's `ComponentRegistry` is a **runtime** value holding
`Box<dyn ComponentFactory>` trait objects. A code generator cannot consult it:
`CardProps::contract()` is a function that runs when the built crate runs, and
codegen runs while that crate is still being compiled. So the binding does not
ask a registry what `Card` is — it computes the id from the **name**, exactly
as `#[derive(Contract)]` does at expansion time (the derive calls straight into
[`component_id_for`], so there is one definition rather than two that agree by
luck). A host that registers the derived contract's id and an author who writes
the tag therefore meet on the same handle without either knowing about the
other.

What this does **not** do is make an unregistered tag loud. `<Crad/>` (a typo)
binds to `user_component_id("Crad")`, nothing is registered under it, and it
mounts nothing — no diagnostic. Nor is it even visibly different: no part of
matching, cascade, layout or paint reads a `ComponentId` (only
`ComponentStore::mount` does), so a typo'd component tag still renders as the
same box it silently rendered as before. What it can no longer do is collide
with [`CONTAINER`] and mount whatever a host registered there.

Making it loud is the half this does not close. The file front end could
(its `Compiler` already takes a `ContractRegistry`); the `okui!` macro cannot,
for the reason above — so the rule cannot be stated once for both front ends,
and stating it for one is the drift the `<VirtualFor>`-was-`<For>` finding was
written against. `okui_user_component.rs`'s
`an_unregistered_component_tag_mounts_nothing` pins the residue.

# Drift

The raw handle values below are duplicated from their canonical definitions
(`obscur-view` reserved symbols; the widget/primitive `COMPONENT_ID`
constants) because this frontend crate must not depend on the component tier.
A drift-guard test in the facade (`obscur-kit`) asserts every constant here
still equals its canonical source.
