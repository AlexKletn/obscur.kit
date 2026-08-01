Produces the component's desired [`View`] for the given `props`.

This is a pure description of the intended output: it returns plain view
data and performs no rendering, reconciliation, or dispatch itself. Use
`ctx` to read the lifecycle phase, open scoped subscriptions, or
[`request_update`](ComponentContext::request_update).
