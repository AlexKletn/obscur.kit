A builder that produces the toolkit's default backends.

The factory methods are gated on the Cargo features that pull in their
adapter crate, so this type always compiles for the active feature set. Each
method constructs a fresh instance — the caller owns the wiring into a
runtime or host (the platform event loop / GPU device live with the host).
