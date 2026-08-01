The sink a [`ReplaySession`] drives while replaying a recording.

The caller implements this over their deterministic infrastructure
(a [`DeterministicClock`](crate::deterministic::DeterministicClock),
executor, and completion queue). Replay walks the recorded steps in order and
dispatches each to the matching method, so replay runs entirely against
deterministic clocks/executors (spec §7.13).

Only [`apply_input`](Self::apply_input), [`advance_clock`](Self::advance_clock),
and [`complete_async`](Self::complete_async) must be implemented. The
scheduler-decision, resource-version, and environment-change hooks default to
no-ops so a target that does not care about them stays minimal.
