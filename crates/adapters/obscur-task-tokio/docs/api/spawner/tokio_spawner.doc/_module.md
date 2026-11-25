A Tokio-backed [`Spawner`] adapter (spec §2.7 optional task ports —
"Опциональные реализации task ports. Ни одна не является core dependency.").

[`TokioSpawner`] implements the executor-agnostic [`Spawner`] port by
forwarding each [`BoxFuture`] to a Tokio [`Handle`], the runtime the host
chose to bring into its reach. The kit's core never names Tokio; this adapter
is one concrete way a host wires the port.
