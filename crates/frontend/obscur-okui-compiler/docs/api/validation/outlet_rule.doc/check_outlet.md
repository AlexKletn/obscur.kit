Checks an authored `<slot …/>` against the §2.12 outlet rule, returning one
message per violation.

Empty for every tag that is not an outlet, and for a well-formed
`<slot name="…"/>`. Every violation is reported rather than just the first,
the same report-them-all policy `Compiler::compile` and the `okui!` macro's
`validate_element` follow.
