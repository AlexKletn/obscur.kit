A single version-to-version transform over a UI IR document (spec §2.5).

A migration knows the [`from_version`](UiIrMigration::from_version) it
consumes and the [`to_version`](UiIrMigration::to_version) it produces, and
rewrites a document from the former schema to the latter via
[`migrate`](UiIrMigration::migrate) (which takes the document by value and
returns the migrated document). Real migrations override `migrate` to
rewrite the node tree and its properties for the new schema; see
[`IdentityMigration`] for the trivial version-bump-only reference.

The trait is object-safe — migrations are stored as `Box<dyn UiIrMigration>`
in a [`MigrationRegistry`](crate::migration::MigrationRegistry) — and
requires a [`std::fmt::Debug`] supertrait so a boxed migration is itself
`Debug`.
