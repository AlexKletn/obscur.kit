A web [`NotificationProvider`] backed by the browser `Notification` API
(spec §6.12).

The provider records the last app-icon badge value set through
[`Self::set_badge`]. Surfacing it via the browser Badging API
(`navigator.setAppBadge`) is a documented future refinement; for now the badge
is stored but not displayed. [`Self::permission`] reads the browser's current
`Notification.permission`, and [`Self::post`] presents a notification only when
permission is already granted.
