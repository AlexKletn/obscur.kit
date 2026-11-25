A reference, no-op [`AccessibilityBridge`] for headless or unsupported
platforms (spec §7.1).

`NullAccessibilityBridge` owns nothing platform-specific: every downward push
([`update_tree`](AccessibilityBridge::update_tree),
[`apply_update`](AccessibilityBridge::apply_update),
[`set_focus`](AccessibilityBridge::set_focus),
[`announce`](AccessibilityBridge::announce)) is a no-op, and
[`poll_actions`](AccessibilityBridge::poll_actions) returns nothing because
no assistive technology is attached. Its silence is correct behavior for a
platform without an accessibility API, not a stub awaiting implementation.
