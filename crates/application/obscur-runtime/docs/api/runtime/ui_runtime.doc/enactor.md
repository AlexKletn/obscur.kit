Enacts each frame's root reconcile through the atomic transaction commit
boundary (spec §3.2): it stages the reconcile as a validated
[`UiTransaction`](crate::transaction::UiTransaction), commits it, and
advances the commit [`VersionStamp`](obscur_identity::VersionStamp) when
the tree actually changed — so the live tree mutation flows through the
transaction layer rather than calling `reconcile_root` directly.
