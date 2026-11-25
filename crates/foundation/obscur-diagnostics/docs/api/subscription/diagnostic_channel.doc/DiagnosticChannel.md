A multi-subscriber diagnostics channel (spec §7.8).

Consumers [`subscribe`](Self::subscribe) with a [`DiagnosticFilter`] and a
[`DiagnosticBufferPolicy`]; each subscription buffers independently, so
backpressure is per-subscription. [`publish`](Self::publish) offers a
diagnostic to every enabled, matching subscription. Buffered diagnostics are
consumed with [`drain`](Self::drain).
