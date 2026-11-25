Sets the Wayland `app_id` (and X11 `WM_CLASS`) the window reports, defaulting
to `"obscur-kit"`.

This is the identity a desktop environment matches on — a compositor's rules
(Hyprland's `class`, sway's `app_id`), the taskbar, and `.desktop` file
association all key off it, **not** off the human-readable
[`title`](Self::new). Give each application a distinct one so a rule can
target it alone.
