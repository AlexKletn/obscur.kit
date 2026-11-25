Per-phase frame budgets: [`FrameBudget`].

Spec §7.9 requires that exceeding a performance budget produce a diagnostic,
not a crash. The runtime is deliberately **clock-free** (the scheduler and
frame loop observe no wall-clock; determinism is a hard requirement), so a
frame phase has no wall-clock duration to budget. Instead each phase reports
a deterministic *cost count* — the number of nodes it processed, boxes it
arranged, or paint commands it produced — and this budget bounds that count.

To reuse the tested [`PerformanceBudget`]/[`BudgetViolation`] machinery
(which is expressed in [`Duration`]), a cost count is encoded as a duration
at the rate **one item = one nanosecond**. The encoding is purely a carrier:
[`BudgetViolation::overage`](obscur_diagnostics::BudgetViolation::overage) and
[`ratio`](obscur_diagnostics::BudgetViolation::ratio) then read back as item
counts and count ratios, and the emitted
[`Diagnostic`](obscur_diagnostics::Diagnostic) flows through the normal
diagnostics channel unchanged.
