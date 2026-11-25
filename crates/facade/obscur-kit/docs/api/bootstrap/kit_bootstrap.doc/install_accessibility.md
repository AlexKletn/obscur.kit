Injects the AccessKit accessibility bridge into `runtime` so every frame
publishes its semantic tree, and returns the shared bridge the host
publishes *from* (spec §7.1).

This is the §7.1 output path's kit half, and it is here for the same reason
[`CssPipeline`](super::CssPipeline) is: joining the runtime's semantic
projection to a platform adapter is fixed plumbing every host needs and
none should hand-write. The facade constructs the concrete adapter and hands
the runtime a domain port, so the application layer depends only on
[`AccessibilityBridge`](crate::accessibility::port::AccessibilityBridge) —
never on this adapter crate. From here on, each frame that changes the
retained tree or moves focus converts a fresh AccessKit `TreeUpdate` into
the returned bridge; nothing in the application ever pushes it.

# What the host must still supply

A host connects the returned bridge to the platform adapter for its
windowing layer (`accesskit_winit`, `accesskit_unix`, …) and works with it during its
event-loop turns:

- feeds [`last_tree_update`](crate::accesskit_bridge::bridge::AccessKitBridge::last_tree_update)
  to that adapter — re-reading it whenever
  [`is_dirty`](crate::accesskit_bridge::bridge::AccessKitBridge::is_dirty)
  or a frame ran;
- drains [`take_announcements`](crate::accesskit_bridge::bridge::AccessKitBridge::take_announcements)
  into the platform's live-region events;
- passes an inbound AccessKit `ActionRequest` to
  [`push_action_request`](crate::accesskit_bridge::bridge::AccessKitBridge::push_action_request);
- wakes the UI loop after an accepted request. Frame entry points drain the
  bridge before checking for a due frame. A custom host can explicitly call
  [`UiRuntime::poll_accessibility_actions`](obscur_runtime::UiRuntime::poll_accessibility_actions).

The runtime routes activation and focus through its usual input handlers and
can scroll a whole node into view. It checks that targets are still live and
valid in the current focus scope. The platform window adapter is supplied by
the host.

Not calling this leaves the runtime's honest-capability default: no semantic
tree is built at all.
