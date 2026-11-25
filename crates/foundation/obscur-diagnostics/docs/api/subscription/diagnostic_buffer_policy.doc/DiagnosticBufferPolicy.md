Buffering and backpressure policy for a [`DiagnosticSubscription`].

A subscription buffers up to `capacity` diagnostics; when full, `overflow`
decides what happens to further arrivals (spec §7.8).

[`DiagnosticSubscription`]: crate::DiagnosticSubscription
