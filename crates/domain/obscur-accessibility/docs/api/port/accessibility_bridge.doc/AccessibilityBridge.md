The replaceable port platform accessibility adapters implement (spec §7.1).

A single trait through which the pure accessibility domain drives the outside
platform accessibility API — UI Automation, AX, AT-SPI, or web a11y. An
adapter implements it once per platform and the domain stores it as
`Box<dyn AccessibilityBridge>`, so the trait is deliberately **object-safe**:
no generic methods and no `Self`-returning methods.

The bridge pushes tree state downward — a full [`SemanticSnapshot`] or an
incremental [`SemanticUpdate`] — reflects [`AccessibilityFocus`] and live
announcements, and drains inbound [`AccessibilityAction`] requests upward so
they can be dispatched through the same command/event model as ordinary
input.
