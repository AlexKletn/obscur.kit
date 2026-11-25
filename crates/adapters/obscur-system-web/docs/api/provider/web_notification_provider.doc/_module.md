A web [`NotificationProvider`] backed by the browser `Notification` API
(spec §6.12).

[`WebNotificationProvider`] implements the pure
[`NotificationProvider`](obscur_system::notification::NotificationProvider)
port over `web-sys`'s [`web_sys::Notification`]. Because the port is
synchronous, this provider only *reads* the current permission and posts when
it is already `Granted`: requesting permission is an asynchronous browser call
(`Notification.requestPermission()`) that must be driven by the application
before using this provider, and is out of scope for this sync port.

All browser calls are gated behind `#[cfg(target_arch = "wasm32")]`; on any
other target the provider reports [`SupportLevel::Unavailable`] and
[`NotificationResult::Unsupported`], keeping the host build honest.
