Expander for `#[derive(Contract)]`.

Call [`ContractDerive::expand`] with the annotated item's tokens to obtain
the generated inherent `impl` (or a [`syn::Error`] for unsupported inputs).
