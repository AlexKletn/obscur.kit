Installs the per-phase cost [`FrameBudget`] checked after each frame phase
(spec §7.9), replacing any previously installed budget.

A phase whose deterministic cost count exceeds its bound records a
[`BudgetViolation`] into the installed telemetry sink on the next frame —
a warning-level diagnostic, not a crash.

[`BudgetViolation`]: obscur_diagnostics::BudgetViolation
