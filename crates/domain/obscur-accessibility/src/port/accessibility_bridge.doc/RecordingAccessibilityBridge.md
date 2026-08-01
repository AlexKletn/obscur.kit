A deterministic recording [`AccessibilityBridge`] for tests (spec §7.1).

`RecordingAccessibilityBridge` observes what the domain pushes — counting
[`tree_updates`](Self::tree_updates) and [`updates_applied`](Self::updates_applied),
retaining the [`last_announcement`](Self::last_announcement) and
[`last_focus`](Self::last_focus) — and lets a test seed inbound requests with
[`queue_action`](Self::queue_action) so
[`poll_actions`](AccessibilityBridge::poll_actions) drains them in order. It
makes the accessibility command loop testable without any real platform.
