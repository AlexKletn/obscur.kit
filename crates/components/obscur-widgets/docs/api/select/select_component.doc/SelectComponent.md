The shipped `select` as a mounted component.

It is a unit type, and for this control that is a stronger claim than it was
for the checkbox: a select *looks* like it holds state — a popup that opens, a
choice that is remembered — and it does not. Both facts are properties of the
host node the runtime is rendering, written there by whoever owns the model,
so every frame's output is a pure function of the props. What
[`Select::open`] and [`Select::select`] mutate is the caller's own [`Select`]
value, and the node they produce is the statement.

It renders the control's **children**, not the control: the runtime reconciles
a component's output *into* its host node, and the host is already the
`<select>` element — authored, or built by [`Select::view`]. Returning the
whole widget view would nest a second select inside the first.

[`Select`]: super::Select
[`Select::open`]: super::Select::open
[`Select::select`]: super::Select::select
[`Select::view`]: super::Select::view
