Maps this outcome to the [`RecoveryAction`] reported to diagnostics.

- [`Retried`](Self::Retried) → [`RecoveryAction::Retried`]
- [`FellBack`](Self::FellBack) → [`RecoveryAction::BoundaryReplaced`]
- [`Propagated`](Self::Propagated) → [`RecoveryAction::None`] (no
  recovery happened at this boundary; the failure was handed outward)
