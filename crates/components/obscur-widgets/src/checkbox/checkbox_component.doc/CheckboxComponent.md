The shipped `checkbox` as a mounted component.

It is a unit type, and that is a claim rather than an omission: a checkbox
holds no state its host node does not already carry. Its label, its checked
value and its disabled flag are all properties of the node the runtime is
rendering, so every frame's output is a pure function of the props — there
is nothing for an instance to remember. A control that *does* keep private
state (a text field's caret, a select's open popup) will hold it here.

It renders the control's **children**, not the control: the runtime
reconciles a component's output *into* its host node, and the host is
already the `<checkbox>` element — authored, or built by
[`Checkbox::view`]. Returning the whole widget view would nest a second
checkbox inside the first.

[`Checkbox::view`]: super::Checkbox::view
