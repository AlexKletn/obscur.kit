Enqueues an inbound assistive-technology `action` for the domain to
drain via [`poll_actions`](AccessibilityBridge::poll_actions).

The host calls this after translating a platform
[`accesskit::ActionRequest`] into a domain [`AccessibilityAction`].
