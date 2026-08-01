Dispatches `input` against `runtime`, returning the typed outcome.

An input whose target does not resolve — an unknown `test-id`, a node that
is no longer live — is a no-op that reports
[`DispatchOutcome::unresolved`]; it never panics and never guesses a
different target.
