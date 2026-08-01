The validator this enactor gates its own commits with.

Exposed so another commit path — a
[`ProvisionalTransaction`](super::ProvisionalTransaction) being committed,
say — is checked against **this** rulebook rather than a second one it
constructed for itself. The validator is stateless, so sharing it costs
nothing; the point is that there is one set of rules, and no way for a
caller to drift from it.
