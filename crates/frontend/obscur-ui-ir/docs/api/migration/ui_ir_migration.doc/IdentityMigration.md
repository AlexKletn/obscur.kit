The trivial reference migration: a pure version bump that leaves the tree
untouched (spec §2.5).

[`migrate`](UiIrMigration::migrate) returns the document unchanged except for
stamping it with [`to`](IdentityMigration::to). It is useful for versions
whose schema did not actually change, and as a template real migrations
build on by overriding `migrate` to rewrite nodes and properties.
