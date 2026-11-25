The desired node's **full** property list, copied during reconciliation.

[`attributes`](Self::attributes) above is only the CSS-relevant projection
of these (local name, id, class list, boolean states); every other authored
property — a component's `label`, `placeholder`, or any custom prop — lives
only here. Values flow *down* (spec §3.1), and a component's typed props
are decoded from this list after the frame's reconcile, so it is retained
for exactly the reason [`event_bindings`](Self::event_bindings) is: the
runtime must be able to look it up later, once the desired tree is gone.

Empty for non-element nodes and elements that carry no properties.
