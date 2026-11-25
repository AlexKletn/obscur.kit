The immutable record produced when a [`UiTransaction`] is committed atomically
at a frame boundary (spec §3.2).

It carries the normalized operations that were applied together with the
[`VersionStamp`] stamped at commit time. The frame's tree mutation is enacted
through this record: the [`ReconcileEnactor`] stages a reconcile as a
validated [`UiTransaction`], applies it, and produces this commit — the single
authoritative hand-off between staging and application, which is what makes
the commit *atomic* (either the whole batch is recorded here or none of it is)
and *versioned* (the stamp advances only when the tree actually changed). Both
the [`FramePipeline`] and the [`UiRuntime`] drive their per-frame reconcile
through that enactor, so this record is produced on the live frame path rather
than only in unit tests.

[`UiTransaction`]: crate::transaction::UiTransaction
[`ReconcileEnactor`]: crate::transaction::ReconcileEnactor
[`FramePipeline`]: crate::frame::FramePipeline
[`UiRuntime`]: crate::UiRuntime
