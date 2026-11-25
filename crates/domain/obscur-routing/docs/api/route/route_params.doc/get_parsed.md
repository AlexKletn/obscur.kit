Returns the value for `key` parsed into `T` via [`FromStr`] (spec §6.13
typed params).

Returns `None` when the key is absent *or* when the stored string fails
to parse as `T`; the two failure modes are intentionally
indistinguishable, since a caller asking for a typed value only cares
whether a valid `T` is available.
