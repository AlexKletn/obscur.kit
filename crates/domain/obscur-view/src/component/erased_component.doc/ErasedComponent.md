The object-safe face of a [`Component`], so a runtime can hold many
differently-typed components behind one handle and render them (spec §2.1,
§2.14).

[`Component`] is deliberately **not** object-safe: it has an associated
`Props` type, which is the whole point — props stay strongly typed. That
leaves a gap the runtime cannot cross on its own, because a retained node
carries only an untyped `&[ViewProperty]`. This trait is the crossing, and it
is the *only* place the erasure happens.

# You do not implement this

There is a blanket impl for every `C: Component` whose `C::Props: FromProps`,
so deriving `Contract` on the props struct is enough to make a component
erasable. The blanket impl decodes the properties into `C::Props` and then
calls the author's own typed [`render`](Component::render) — the component
never sees an untyped value, and the runtime never sees a typed one.

# Where local state lives

Nowhere here. A component owns its [`LocalState`](crate::state::LocalState)
as plain fields on `Self`, so keeping the same erased value alive across
frames is exactly what preserves that state; there is no store to consult and
the runtime never reaches in (spec §2.1, §3.1).
