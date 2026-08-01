A registry of [`UiIrMigration`]s that applies a chain of them to bring a
document up to a target version (spec §2.5).

Migrations are [`register`](MigrationRegistry::register)ed in any order;
[`migrate`](MigrationRegistry::migrate) starts at the document's own version
and repeatedly applies the migration whose `from_version` matches the current
version, advancing until it reaches the target. The search is bounded so a
cyclic or non-advancing set of migrations fails with
[`MigrationError::NoPath`] rather than looping forever.
