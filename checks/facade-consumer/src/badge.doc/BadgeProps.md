`Badge`'s props (spec §2.13).

`#[contract(name = "Badge")]` is what ties this Rust type to the authored tag:
the derive hashes that string into the component id, and the front end hashes
the same string when it meets `<Badge/>`. Nothing but agreement on the string
makes the two the same component.
`#[contract(style_scope = open)]` is §4.15's "explicitly open/global scopes
are possible", and it is here for two reasons. It is another *path* the
derive emits — `::obscur_kit::view::component::StyleScope` — that only a
crate outside this workspace can prove resolves. And it is what makes the
component's output part of the document for styling: scoped is the default,
and a scoped component's internals are reachable by nothing but a `::part()`
export, which is the state audit finding 2-1 described as making the tier
unusable for anything visual. An application that wants to style what it
wrote says this, once, where it declares the component.
