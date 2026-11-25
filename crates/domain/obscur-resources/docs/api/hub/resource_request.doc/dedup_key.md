Returns the identity key the hub deduplicates on: `(source, kind)`
(spec §5.15 deduplication).

The namespace is deliberately excluded — it is ownership metadata, not
identity — so requests for the same bytes-and-category collapse to one
resource regardless of scope.
