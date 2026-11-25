Appends the statement pushing a single `ViewNode::Element` for `node`.

`position` comes from the caller's [`UiIrNodeKind`], which is where the front
ends already recorded their component-vs-element verdict; it decides what an
unrecognised tag falls back to (a `Container` box, or the component's own
name-derived id — see [`name_binding::component_id_for`]) **and** whether the
tag is also emitted as the node's local name (see
[`name_binding::local_name_for`]): an element is its tag, a component is not.
