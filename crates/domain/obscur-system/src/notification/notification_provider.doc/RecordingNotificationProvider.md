A deterministic, recording [`NotificationProvider`] for tests (spec §6.12).

`RecordingNotificationProvider` reports [`SupportLevel::Native`] and accepts
everything: [`post`](Self::post) pushes the notification onto
[`posted`](Self::posted) and returns [`NotificationResult::Delivered`], and
[`set_badge`](Self::set_badge) stores the value in [`badge`](Self::badge).
Tests inspect the recorded state to assert what the domain attempted, without
touching a real notification service.
