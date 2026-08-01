The replaceable port for presenting system notifications (spec §6.12).

[`NotificationProvider`] is the hexagonal port through which the domain asks
the host to [`post`](NotificationProvider::post) a
[`Notification`] and to manage the app-icon badge. Permission and host policy
are **explicit** (spec §6.12): a provider reports its honest
[`permission`](NotificationProvider::permission) as a [`SupportLevel`], and a
refusal is the first-class [`NotificationResult::Denied`] outcome rather than a
panic. The colocated [`NullNotificationProvider`] and
[`RecordingNotificationProvider`] are reference implementations of that
contract.
