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

**This does not by itself reach a screen reader**, and the returned handle
is the reason: AccessKit's platform half needs an OS window, which this kit
never owns. A host completes the path with the platform adapter for its
windowing layer (`accesskit_winit`, `accesskit_unix`, …) and, on each of its
own event-loop turns:

- feeds [`last_tree_update`](crate::accesskit_bridge::bridge::AccessKitBridge::last_tree_update)
  to that adapter — re-reading it whenever
  [`is_dirty`](crate::accesskit_bridge::bridge::AccessKitBridge::is_dirty)
  or a frame ran;
- drains [`take_announcements`](crate::accesskit_bridge::bridge::AccessKitBridge::take_announcements)
  into the platform's live-region events;
- translates each inbound AccessKit `ActionRequest` with
  [`ActionConverter::from_accesskit`](crate::accesskit_bridge::conversion::ActionConverter::from_accesskit)
  and enqueues it with
  [`push_action`](crate::accesskit_bridge::bridge::AccessKitBridge::push_action).

That last one is only half of §7.1's "accessibility actions enter the same
command/event model as ordinary input": the runtime does **not** drain
[`poll_actions`](crate::accessibility::port::AccessibilityBridge::poll_actions),
so an action an assistive technology requests reaches the bridge and stops
there. The inbound path is unbuilt; this installs the outbound one.

Not calling this leaves the runtime's honest-capability default: no semantic
tree is built at all.
