Per-property inheritance classification for the cascade.

[`InheritedProperty`] is a thin, stateless classifier: given a
[`PropertyId`], [`InheritedProperty::of`] answers whether that property
inherits from its parent by default (§4.14). Modelling this as a classifier
(rather than a stored `(PropertyId, Inheritance)` pair) keeps a single source
of truth — [`PropertyId::default_inheritance`] — and avoids the risk of a
stored pair drifting out of sync with the property's real default.
