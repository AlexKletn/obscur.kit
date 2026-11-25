Migrates `doc` from its current version up to `target`.

Starting at `doc.version()`, repeatedly applies the migration whose
`from_version` equals the current version, advancing the version each
step, until the version equals `target`. If the document is already at
`target` it is returned unchanged. If no registered migration advances
the current version before reaching `target`, returns
[`MigrationError::NoPath`].

The step count is bounded by `migrations.len() + 1` so a migration whose
`to_version == from_version` (or any cycle) cannot loop forever; on
exceeding the bound the same `NoPath` error is returned.

# Errors

Returns [`MigrationError::NoPath`] when no chain of registered migrations
reaches `target`.
