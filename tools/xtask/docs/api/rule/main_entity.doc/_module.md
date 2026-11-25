Checks the one-entity-per-file rule in workspace crate source directories.
File-scope structs, enums, unions, and traits count regardless of visibility.
Implementations, functions, and unit-test modules may accompany the entity.

A deliberate exception needs both a `// xtask:allow(multiple-entities)`
comment in the source and a workspace-relative entry in [`KNOWN`]. The checker
rejects stale or incomplete exceptions.
