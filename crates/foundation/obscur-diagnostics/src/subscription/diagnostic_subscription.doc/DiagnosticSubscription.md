A live subscription to the diagnostics channel (spec §7.8).

A subscription admits diagnostics matching its [`DiagnosticFilter`], buffers
them under its [`DiagnosticBufferPolicy`] (with per-subscription
backpressure), and can be independently disabled. Buffered diagnostics are
consumed with [`drain`](Self::drain).
