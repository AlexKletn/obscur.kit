Lowers an element/component IR node into a single [`ViewNode::Element`].

Names bind through the shared [`name_binding`] registry: the tag resolves to
a real [`ComponentId`](obscur_identity::ComponentId) (a built-in widget or
primitive; else the generic `Container` for an element, or the component's own
name-derived id for a component), reserved attributes/events resolve to the
`obscur-view` symbols the reconciler/matcher/dispatcher read, and the handler
command resolves to a deterministic hashed symbol.

An element tag is *also* the node's local name (the §4.13 type selector's
input), which is what makes `<button>` a base element rather than an anonymous
box; see [`name_binding::local_name_for`].

`position` comes from the caller's [`UiIrNodeKind`] rather than from the tag's
spelling: the front ends already decided component-vs-element and recorded it
on the node, and re-deriving it here would be a second copy of their rule.
