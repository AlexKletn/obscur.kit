Derives the semantic view for this button: a [`COMPONENT_ID`] element
carrying the role, disabled, and pressed state as properties, a click
event bound to `command`, and the label as its *contents* — a single
[`Text`] child, or none when the label is empty.

The label is a **child** and not a property, which is what makes this node
the same thing an authored `<button>Save</button>` is. A mounted
[`ButtonComponent`] renders the default-slot outlet, so the child put here is
projected straight back into it and the node the runtime keeps is the node
this built.

[`COMPONENT_ID`]: Self::COMPONENT_ID
[`ButtonComponent`]: super::ButtonComponent
