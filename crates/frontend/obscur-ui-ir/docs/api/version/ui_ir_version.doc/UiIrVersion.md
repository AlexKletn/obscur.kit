The format version of a UI intermediate-representation document (spec §2.5).

A `UiIrVersion` is a `major.minor` pair identifying the IR schema a document
was written against. It follows a semver-style compatibility rule:

- The **major** component gates breaking changes: documents with different
  majors are mutually incompatible.
- The **minor** component gates backward-compatible additions: a reader can
  read any document whose minor is less than or equal to its own, as long as
  the majors match.

See [`UiIrVersion::is_compatible_with`] for the exact rule. Ordering is
lexicographic by `major` then `minor`.
