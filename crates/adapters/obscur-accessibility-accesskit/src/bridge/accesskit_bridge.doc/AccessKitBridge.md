A host-driven [`AccessibilityBridge`] backed by AccessKit (spec §7.1, §2.7).

This adapter **converts** the domain semantic tree into AccessKit types and
**stores** the latest [`TreeUpdate`]; it does not own a platform accessibility
connection itself. The platform push lives in the host: an `accesskit_winit`
or `accesskit_unix` adapter belongs to the windowing host, which feeds this
bridge's [`last_tree_update`](Self::last_tree_update) into that platform
adapter and, in the opposite direction, converts each inbound
[`accesskit::ActionRequest`] (via
[`ActionConverter::from_accesskit`](crate::conversion::ActionConverter::from_accesskit))
into an [`AccessibilityAction`] it enqueues with [`push_action`](Self::push_action).
Keeping the platform loop in the host preserves the adapter layering
(`adapter → foundation + domain port + external implementation`) and keeps
this crate free of any OS or event-loop dependency.

The determinism boundary of spec §7.1 lives in the conversion: identical
snapshots always produce identical [`TreeUpdate`]s regardless of platform.
