The default, authoritative contract set for the built-in view primitives
([`builtin_contracts`]) and the shipped widgets ([`widget_contracts`])
(spec §2.6, §2.13).

Spec §2.13 requires that unknown attributes and events on a *known*
component are compile errors. For that check to bite on a real compile, the
[`ContractValidator`](super::ContractValidator) must consult a **populated**
registry rather than an empty one; this module supplies both sets so the
default compile path enforces their attribute/event surface out of the box.

# Why the surface is mirrored here rather than imported

The primitives live in `obscur-primitives` as plain view-builder config
structs and deliberately **do not** export a component contract (they carry
only stable [`ComponentId`]/prop-symbol constants). The typed
`obscur_view::component::ComponentContract` names every prop/event by an
opaque, hash-derived `InternedSymbol` with no reverse string resolution, so
it cannot drive the `.okui` string-keyed lookup either. The authoritative
*string* surface for these primitives is therefore their spec-defined
builder API (spec §2.6): each contract below mirrors one primitive's
attributes/events exactly.

[`ComponentId`]: obscur_identity::ComponentId
