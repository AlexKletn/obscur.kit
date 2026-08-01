A bound on each frame phase's deterministic cost count (spec §7.9).

A `None` limit means the phase is unbounded (the default): no violation is
ever produced for it. A `Some(limit)` phase yields a [`BudgetViolation`] when
the phase's reported count strictly exceeds `limit`.

Budgets are attributed to the same subsystems the frame phases record spans
under: reconcile → [`Tree`](DiagnosticSubsystem::Tree), style →
[`Style`](DiagnosticSubsystem::Style), layout →
[`Layout`](DiagnosticSubsystem::Layout), compose →
[`Render`](DiagnosticSubsystem::Render).
