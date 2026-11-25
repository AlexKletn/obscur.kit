A record that a measured duration exceeded a [`PerformanceBudget`].

Exceeding a budget is a recoverable concern (spec §7.9): it produces a
warning-level [`Diagnostic`] via [`to_diagnostic`](Self::to_diagnostic)
rather than a crash, so it flows through the normal diagnostics channel.

[`PerformanceBudget`]: crate::PerformanceBudget
