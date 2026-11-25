Returns the inherited computed value for `property`: the parent's computed
value, or the property's initial value when there is no parent.

When a parent exists but did not compute `property` (a deferred keyword
initial), the result is [`None`] and the property is left unresolved.
