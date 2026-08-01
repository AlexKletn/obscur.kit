Recovery for non-component subsystem failures — layout, renderer, resources
— which use subsystem-specific recovery boundaries (spec §1.7).

These subsystems have no component [`ErrorBoundary`](obscur_view::boundary::ErrorBoundary),
so recovery is coarser than component recovery: there is no fallback *view*,
only *retry-or-degrade*. A subsystem failure is either retried while budget
remains or the subsystem degrades — keeping its last-good output where it can
— and it never [`Propagated`](RecoveryResult::Propagated)s to a component
boundary.

The policy is a single retry budget shared across subsystems. Phase 1 treats
all subsystems uniformly; the [`DiagnosticSubsystem`] is accepted for future
per-subsystem policy and diagnostics attribution.
