Creates a source with no registered snapshots.

Until a payload is registered with [`with_snapshot`](Self::with_snapshot)
or [`set_snapshot`](Self::set_snapshot), every target resolves to the
typed [`default_payload`] rather than to `Null`'s placeholder.
