Validates the pending transaction and, on success, produces a
[`TransactionCommit`] stamped with `version`.

This is the atomic commit path: validation happens first, and the commit
record is produced only if validation passes. On failure the
[`TransactionError`] is returned and nothing is applied — the staged
operations never touched any domain.

# Errors

Returns a [`TransactionError`] when validation of the pending transaction
fails; in that case nothing is committed.
