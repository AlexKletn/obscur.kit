The host-driven AccessKit [`AccessibilityBridge`] implementation (§7.1).

[`AccessKitBridge`] converts the domain semantic tree into AccessKit types
and stores the latest [`accesskit::TreeUpdate`] for a windowing host to feed
into a platform adapter (`accesskit_winit` / `accesskit_unix`), keeping this
crate free of any OS dependency.

[`AccessibilityBridge`]: obscur_accessibility::port::AccessibilityBridge
