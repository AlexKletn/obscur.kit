Installs the per-phase cost [`FrameBudget`] checked after each frame phase
(spec §7.9), replacing any previously installed budget.

A phase whose deterministic cost count exceeds its bound records a
[`BudgetViolation`] with the telemetry sink and emits a warning through the
structured diagnostic sink.

[`BudgetViolation`]: obscur_diagnostics::BudgetViolation
