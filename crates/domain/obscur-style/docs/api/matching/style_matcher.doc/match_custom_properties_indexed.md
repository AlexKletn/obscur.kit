Collects `element`\s custom properties the way
[`match_custom_properties_in`](Self::match_custom_properties_in) does, but
consults `index` for the rules worth trying.

Same result, same cascade, same second-pass semantics: a rule declaring no
custom property never advances the counter, so the numbers this stamps are
the ones the unindexed pass stamps. A sheet set that declares no custom
property at all is answered without touching the element.

Falls back to the unindexed pass when `index` does not describe `sheets` or
the element cannot enumerate its classes.
