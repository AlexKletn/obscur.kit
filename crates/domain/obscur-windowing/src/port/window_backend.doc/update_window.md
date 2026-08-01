Applies the dynamic property delta `changes` to its target window
(spec §6.2).

The target window is taken from [`UpdateWindow::target`]; there is no
separate `id` parameter, so the request is the single source of truth and
an id/target mismatch is impossible by construction. Reports the honest
[`WindowOperationResult`] — including the [`SupportLevel`] at which the
property changes were satisfied — and returns
[`WindowOperationResult::Failed`] for an unknown target rather than
panicking.
