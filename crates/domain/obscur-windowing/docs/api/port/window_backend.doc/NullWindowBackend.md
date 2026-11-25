A reference, no-op [`WindowBackend`] that owns nothing platform-specific
(spec §6.1-6.6).

`NullWindowBackend` retains created [`Window`] records in memory, keyed by a
monotonically increasing [`WindowId`], and honestly reports that it supports
no real platform capability: activation is
[`ActivationResult::Unsupported`], every chrome/presentation/sleep operation
is [`WindowOperationResult::Unsupported`], [`Self::supports`] is
[`SupportLevel::Unavailable`] for every feature, and it enumerates no
displays and emits no events. It exists to exercise the port and to document
the honest-capability contract concretely — its "unsupported" answers are
correct behavior, not stubs waiting to be filled in.
