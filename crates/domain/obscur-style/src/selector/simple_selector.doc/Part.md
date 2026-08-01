An exported style part such as `::part(label)`; stores the part name
(no `::part(` / `)`), §2.12 / §4.15.

Like [`PseudoElement`](Self::PseudoElement) it is inert in the ordinary
single-element matcher: `Component::part(name)` matches an element in a
component's *scoped* render output that the component exports under
`name`, and deciding that needs the component-host boundary — tree
context the pure matcher does not have. The runtime's style-matching
coordinator matches it against the exported part while the rest of the
compound matches the host.
