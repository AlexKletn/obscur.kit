The outcome of resolving a [`FocusRequest`](super::FocusRequest) (spec §7.1).

The focus state machine returns exactly one of these for every request,
distinguishing a successful move from a rejection (target exists but is not
focusable), no available target, or a request that left focus unchanged.
