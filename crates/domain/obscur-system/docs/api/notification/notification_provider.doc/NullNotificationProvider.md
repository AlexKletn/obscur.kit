A reference, no-op [`NotificationProvider`] that presents nothing
(spec §6.12).

`NullNotificationProvider` honestly reports [`SupportLevel::Unavailable`]:
[`post`](Self::post) returns [`NotificationResult::Unsupported`] and
[`set_badge`](Self::set_badge) is a no-op. Its answers are correct behavior — a
documented reference for the honest-capability contract — not a stub waiting to
be filled in.
