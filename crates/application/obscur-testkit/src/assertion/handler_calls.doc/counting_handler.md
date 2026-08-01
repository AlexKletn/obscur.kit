A counter and a handler that bumps it.

Returned as a pair because the two halves go to different places: the handler is
given away to the runtime (`on_command`, `on_command_at`, `on_scope_command`,
`add_event_listener`, an `AppCommands` fallback — every link of §2.6's chain
takes the same shape), and the counter stays with the test, which reads it to
say *how many times* the handler ran rather than merely that it did.

The count, not the boolean, is what the routing tests are written against: a
chain that tried two links would run one handler twice, and a `bool` could not
tell that from the correct single claim.

Its signature names nothing from a crate that is built twice, so one definition
serves both the unit tests inside `obscur-runtime` and the integration tests
beside them.
