Checks `expr_text` against §2.10, returning one diagnostic per violation,
anchored at `offset`.

Text that is not a Rust expression at all is itself a diagnostic: an
`.okui` binding lowers to Rust, so anything rustc could not parse could
never have run — reporting it here beats emitting it and letting the error
surface inside generated code.

`offset` is the expression's byte offset in the source and is attached to
every diagnostic; resolving a column *within* the expression would need the
span mapping this text-based seam does not carry.
