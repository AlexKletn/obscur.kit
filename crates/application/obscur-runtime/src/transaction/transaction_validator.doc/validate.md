Validates that `tx` is structurally well-formed without mutating it.

# Errors

Returns `Ok(())` if the batch may be committed, or the first
[`TransactionError`] encountered otherwise. Because validation is
side-effect free, a rejected transaction leaves every domain untouched.
