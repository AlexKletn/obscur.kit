Expands `#[derive(Contract)]` on a props struct into an inherent impl
producing an obscur-view `ComponentContract`, named through the facade as
`::obscur_kit::view::component::ComponentContract` (see the module docs).

Only structs with named fields are supported; enums, unions and
tuple/unit structs produce a [`syn::Error`] at the input span. Unknown
`#[contract(...)]` keys are also errors (spec: unknown attributes are
compile errors).

# Errors

Returns a [`syn::Error`] when the input cannot be parsed as a
[`syn::DeriveInput`], when the item is not a named-field struct, or when
an unrecognised `#[contract(...)]` attribute or value is encountered.
