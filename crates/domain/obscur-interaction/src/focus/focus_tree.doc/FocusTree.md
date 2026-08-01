The live focus state: the registered focus nodes and which one is focused
(spec §7.1).

This is the pure focus state machine. [`apply`](Self::apply) resolves a
[`FocusRequest`] against the registered [`FocusNode`]s and the current focus,
returning a [`FocusResult`] and updating the focused node. Tab navigation
uses a [`TabOrder`] computed over the registered nodes in deterministic
[`NodeId`] order. Scope containment and focus traps are *not* enforced here:
the runtime layers those on top using
[`FocusScope`](super::FocusScope) and [`FocusRestore`](super::FocusRestore).
