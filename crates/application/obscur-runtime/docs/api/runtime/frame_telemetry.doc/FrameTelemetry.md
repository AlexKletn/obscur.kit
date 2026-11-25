The §7.9 performance channel: the [`TelemetrySink`] a frame's phases record
into, and the [`FrameBudget`] their cost counts are checked against.

# Why these two are one entity

A phase's deterministic cost count has **two destinations and one meaning**:
it is the value of the counter the phase records, and it is the number the
budget is checked against, and the resulting [`BudgetViolation`] is recorded
into the same sink under the same subsystem. Held apart, that arrangement was
written out at each of the four phase call sites —
`record_phase(&mut *self.telemetry, "layout", …, count, self.budget.check_layout(count), …)`
— where the count was passed twice and the *choice of bound* was made by hand.
Nothing forced the two counts to be the same number, and nothing forced
`check_layout` to be the check under the `"layout"` span: a phase could report
arranged boxes to the counter and check resolved styles against the budget,
and the result is a violation that contradicts the counter beside it. Wrong
telemetry is the failure nothing downstream cross-checks, because telemetry is
what one would cross-check *with*.

[`record_phase`](Self::record_phase) takes the phase and the count once and
derives the span, the label, the subsystem and the bound from the phase, so
there is no longer a call site that can disagree with itself.

An exceeded budget is recorded by the telemetry sink and emitted as a
warning through the runtime diagnostic sink supplied for that phase.
