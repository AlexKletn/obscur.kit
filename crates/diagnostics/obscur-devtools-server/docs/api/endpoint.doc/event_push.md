Whether a loop that actually drains an [`EventQueue`] and writes the
fan-out is driving this session. Only such a loop sets it — see
[`mark_event_push_serviced`](Self::mark_event_push_serviced) — so it can
never claim more than is true, and a `Subscribe` is denied while it is
`false`.

[`EventQueue`]: crate::transport::EventQueue
