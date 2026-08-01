Runs the shared style → layout → compose tail of a frame over an
already-reconciled `tree` and a resolved set of `inputs`, publishing the
snapshot and returning the [`FrameResult`].

Both [`run`](Self::run) and [`run_matched`](Self::run_matched) call this
after their reconcile step so the deterministic core stays identical; the
two entry points differ only in how they obtain `inputs`.
