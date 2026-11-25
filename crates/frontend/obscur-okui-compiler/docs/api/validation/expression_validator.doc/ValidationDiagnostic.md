A validation issue found while checking a document (spec §2.10, §2.13).

This is the common diagnostic reported by every validator in this module —
the [`ExpressionValidator`], the
[`CapabilityValidator`](crate::validation::CapabilityValidator) and the
[`ContractValidator`](crate::validation::ContractValidator). It pairs a human
readable `message` with the byte `offset` into the source where the issue was
detected, so a caller can render a pointed diagnostic. It is deliberately
untyped (a message string rather than an error enum) because the validation
rules are open-ended and reported uniformly.
