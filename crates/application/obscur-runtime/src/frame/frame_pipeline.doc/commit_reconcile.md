Reconciles `tree` toward `desired` **through** the atomic transaction
commit boundary (spec §3.2) and returns whether the tree changed.

This is the single per-frame tree-mutation point. Rather than calling
[`Reconciler::reconcile_root`] directly, it hands the reconcile to a
[`ReconcileEnactor`], which stages it as a validated
[`UiTransaction`](crate::transaction::UiTransaction), commits it
atomically, and (on a real change) advances the commit version — with the
reconcile itself as the commit's effect. The retained tree and every
downstream result are therefore **identical** to the previous direct call;
only the commit path differs.

The pipeline holds no cross-frame state, so the enactor is constructed per
call; the runtime ([`UiRuntime`](crate::UiRuntime)) owns a
persistent enactor whose version advances across frames.
