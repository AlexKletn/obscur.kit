Decides whether a change to the *parent's* computed style affects this
node, returning the typed reason if so (§4.14).

The node is affected when either:
- it inherited from its parent and `parent_diff` changed at least one
  property that inherits by default (checked with
  [`InheritedProperty::of`] over
  [`StyleDiff::changed_properties`]) — yields
  [`StyleInvalidationReason::ParentStyleChanged`]; or
- it reads custom properties and `parent_diff` reports a custom-property
  change ([`StyleDiff::custom_properties_changed`]) — yields
  [`StyleInvalidationReason::CustomPropertyChanged`].

The inherited-property check is preferred when both apply. Returns `None`
when the node consumed nothing the parent changed, so an unaffected node is
never restyled.
