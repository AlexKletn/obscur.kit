A Linux [`NotificationProvider`] that posts via `notify-send` (spec §6.12).

[`LinuxNotificationProvider`] implements the pure
[`NotificationProvider`](obscur_system::notification::NotificationProvider)
port by shelling out to `notify-send` with [`std::process::Command`] and no
external crates. A [`Notification`](obscur_system::notification::Notification)
maps to a `notify-send` argv (see [`build_args`]); the provider spawns the tool
fire-and-forget. `notify-send` failing to spawn usually means no notification
daemon is running, which maps to [`NotificationResult::Denied`].

Linux has no standard per-application icon badge, so [`Self::set_badge`] only
records the value — surfacing it (for example on the Unity launcher) would need
a D-Bus round-trip and is a future refinement (spec §2.7 target-specific
providers).
