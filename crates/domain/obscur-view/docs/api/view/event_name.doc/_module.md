Well-known event-name symbols carried on element event bindings.

An element view node binds an interaction event to a semantic command via a
[`ViewEventBinding`](super::ViewEventBinding) — `event → command`. The
*event* side of that binding is an [`InternedSymbol`], and both the
view-builder that authors the binding and the runtime that fires it must name
the same event. Rather than route these through an interner (which would need
the two sides to share a table), the toolkit reserves a small set of fixed
event-name symbols here, so the authored `on-click` binding and the runtime's
click lookup agree by *value* without any interner coherence.

The reserved names occupy a low symbol namespace (`0x0000_01xx`), distinct
from the style-attribute namespace (`0x0000_00xx`, see
[`style_attribute`](super::style_attribute)) and the component/primitive prop
namespaces (e.g. `0x00C1_xxxx`), so an event name never collides with a style
attribute or an ordinary prop.

Only the *event* is reserved; the *command* a binding maps to stays
app-owned — the application interns its own command symbol and uses the
**same** symbol when it registers the handler with the runtime, which only
compares symbols.
