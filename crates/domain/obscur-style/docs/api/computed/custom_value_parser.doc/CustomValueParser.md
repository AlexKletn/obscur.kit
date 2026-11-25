Parses a custom property's declared text for the property it is substituted
into.

Implemented by whatever layer owns the CSS grammar (in this workspace the
frontend's compiler, wired in by the facade) and handed to the resolver
through
[`ResolutionContext::with_custom_values`](super::ResolutionContext::with_custom_values).

A blanket implementation covers every
`Fn(PropertyId, &str) -> Option<PropertyValue>`, so a boxed closure *is* a
parser and installing one needs no adapter type.
