Validates that every capability an element declares it requires has been
granted (spec §2.13).

# Declaration convention

An element declares a required capability via a `requires="cap.id"` **string**
attribute — for example `<Camera requires="camera"/>`. The validator holds
the set of *granted* capability ids and reports an element whose declared
requirement is absent from that set. A dynamic `requires={..}` expression is
not a static capability id and is ignored (it has no `string_value`).

The granted set is stored as a [`BTreeSet`] of ids so membership checks are
ordered and deduplicated.
