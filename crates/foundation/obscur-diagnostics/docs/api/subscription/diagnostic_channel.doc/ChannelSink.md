A [`DiagnosticSink`](crate::DiagnosticSink) wrapper over a channel, so
existing sink-based producers can publish into a channel.

The channel is held behind interior mutability because [`DiagnosticSink`]
takes `&self`; a `RefCell` keeps the port std-only and unsafe-free.

[`DiagnosticSink`]: crate::DiagnosticSink
