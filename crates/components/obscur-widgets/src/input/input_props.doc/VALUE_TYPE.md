Opaque type identity of the `value` prop: the FNV-1a-32 hash of the Rust type
spelling `String`, which is what `#[derive(Contract)]` records as a
`PropContract`'s `value_type`. It names no view property and is compared,
never interpreted.
