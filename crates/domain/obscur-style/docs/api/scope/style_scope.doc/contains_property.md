Returns `true` when this scope *contains* the given property — i.e. the
property does not escape the boundary.

A `Global` or `Open` scope contains nothing: it performs no isolation, so
every property is visible outside it. An isolated (`Component` /
`Module`) scope contains exactly the properties that do not cross the
boundary — every non-inherited, non-custom property (§4.15).
