AccessKit [`AccessibilityBridge`](obscur_accessibility::port::AccessibilityBridge)
adapter (spec §7.1 accessibility tree, §2.7 "Преобразует semantic
tree/actions в AccessKit integration").

[`AccessKitBridge`](bridge::AccessKitBridge) implements the domain
`AccessibilityBridge` port by converting the semantic
[`SemanticSnapshot`](obscur_accessibility::tree::SemanticSnapshot) into
AccessKit [`TreeUpdate`](accesskit::TreeUpdate) / [`Node`](accesskit::Node) /
[`Role`](accesskit::Role) / [`Action`](accesskit::Action) values, and turns
inbound AccessKit action requests back into domain
[`AccessibilityAction`](obscur_accessibility::event::AccessibilityAction)s.

The design is **host-driven**: this crate only produces AccessKit data and
retains the latest update. The actual platform push (`accesskit_winit`,
`accesskit_unix`) lives in the windowing host, so the adapter stays free of
any OS or event-loop dependency while the determinism boundary of spec §7.1
is preserved entirely inside the pure conversion.

The [`conversion`] module holds the stateless converters; [`bridge`] holds
the port implementation; [`error`] holds the adapter's own diagnostic.
