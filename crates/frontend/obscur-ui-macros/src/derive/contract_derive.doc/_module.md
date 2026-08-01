`ContractDerive`: expansion logic for `#[derive(Contract)]` (spec §2.13).

This module turns a props struct into an inherent `impl` exposing a
`contract()` associated function that builds a fully-typed
`ComponentContract`. The macro crate emits leading-`::` paths as tokens, so
the *generated* code names obscur-view's and obscur-identity's types while
this crate depends on neither.

# Which crate the generated paths name

Through the `obscur-kit` facade — `::obscur_kit::view::…` and
`::obscur_kit::identity::…` — for the reason
[`CodegenRoot`](obscur_okui_compiler::codegen::CodegenRoot) spells out for the
*other* macro: a `::`-prefixed path resolves in the extern prelude, which
holds the calling crate's **direct** dependencies, and an application's only
direct dependency is the facade. This derive emitted `::obscur_view::…` and
`::obscur_identity::…` until it was found from outside the workspace (audit
finding 2-2, the residue of 1-A): every `#[derive(Contract)]` in this
repository lives in `obscur-kit`'s own test targets, where both crates are
reachable, so the failing path was never taken here and `#[derive(Contract)]`
— the whole way a component declares its contract — could not be used by an
application at all.

Unlike `ViewCodegen`'s, this root is **not** an argument, and deliberately so:
there is no caller who could pass a different one. `obscur-ui-macros` is
`publish = false` and only `obscur-kit` depends on it, so the sole way to
reach this derive is through the facade's re-export
(`obscur_kit::ui_macros::Contract`) — a crate naming the derive necessarily
has `obscur_kit` in its extern prelude. A `#[contract(crate = …)]` key would
therefore be a knob with no possible turner. A crate that genuinely wants to
stand on `obscur-view` alone does not use the derive: it writes the two
builders by hand (`ComponentContract::new(…).with_prop(…)`,
`ComponentSurface::new(…)`) plus its own `FromProps`, which is public API and
is exactly what obscur-view's own tests do.

# Stable-id scheme

`InternedSymbol` and `ComponentId` are opaque `from_raw` handles, and there
is no compile-time interner available during derive expansion, so every id is
computed at expansion time and only the resulting integer literal is emitted.

There are two *different* schemes here, and the distinction is load-bearing:

- **Names that also exist in the desired view tree** — each prop/event field
  name, the component's own name, and a slot's `child = "Tag"` — are resolved
  through the shared `obscur_ui_ir::name_binding` registry ([`name_symbol`] /
  [`event_name_symbol`] / [`component_id`]), because a contract's handle is
  only useful if it equals the one a real node carries. `name_binding` maps
  reserved names (`class`, `id`, `type`, `src`, `disabled`, `checked`;
  `click`, `input`, `change`) to fixed low constants and everything else to
  `fnv32(name) | 0x8000_0000`; a component name maps to its built-in's
  `COMPONENT_ID` or to `fnv64(name) | 1<<63`. A bare `fnv32`/`fnv64` — which
  this derive used to emit — agrees with that only by luck.
- **Names that exist only inside the contract model** — the component name
  symbol, each field's normalized *type* string, and a §2.12 slot's name —
  have no view-tree counterpart to agree with, so they stay plain FNV-1a
  ([`fnv32`]). A slot name is one of these because a `slot="header"` reaches
  the tree as the *value* of a property, never as a symbol.

# Slots (§2.12)

A struct-level `#[contract(slot(name = "header", cardinality = one))]`
declares one named slot; `cardinality` is `one`/`optional`/`many` (default
`many`) and a repeatable `child = "Tag"` restricts what the slot admits.

Both views are emitted from the one declaration, and only one of them is read
today: the generated `contract_surface()` (a `ComponentSurface`) is what the
`.okui` compiler and the `okui!` macro check a document against, while the
typed `contract()`'s `slots` field has no production reader — as is true of
its props and events. Emitting it anyway is not decoration: a component that
declares a slot and whose typed contract reports `slots: []` would be *wrong*,
and an unread field beats a lying one.

# Style scope (§4.15)

A struct-level `#[contract(style_scope = open)]` declares that the document's
ordinary selectors reach the elements this component renders; `scoped` (the
default, and what an unstated attribute means) keeps them out, so only an
exported `::part()` crosses. This one *is* read in production — the runtime
reads it at mount and stamps `NodeFlags::OPEN_STYLE_SCOPE` on the host, which
is what the style pass's scope walk consults. Without it a component's output
could be styled by nothing at all, which is what the external consumer audit
recorded as finding 2-1.
