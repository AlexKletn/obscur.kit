The fully-resolved computed style of a single node (§4.14).

`values` holds the concrete [`ComputedValue`] for each [`PropertyId`] the
resolver computed for the node. `custom_properties` holds the effective,
inheritable custom properties (parent's, overridden by the node's own) so a
child's resolution can start from them.

Because [`ComputedValue`] and [`PropertyValue`] may carry `f32` data, this
derives [`PartialEq`] but not [`Eq`]/[`Hash`].
