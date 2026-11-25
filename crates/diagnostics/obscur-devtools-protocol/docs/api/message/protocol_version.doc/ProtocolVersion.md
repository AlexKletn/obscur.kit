A protocol version as `{ major, minor }`.

Compatibility follows semantic-version rules for a wire protocol: the same
`major` is required, and the serving side's `minor` must be at least the
peer's (a higher minor is additive and backward compatible).
