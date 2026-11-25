The shipped `radio` as a mounted component.

It is a unit type, and that is a claim rather than an omission: a radio holds
no state its host node does not already carry. Its label, its selection and
its disabled flag are all properties of the node the runtime is rendering, so
every frame's output is a pure function of the props — there is nothing for an
instance to remember. *Which* member of a group is selected is not this
control's state either: §2.6 leaves grouping to the caller, and the node says
which one it is.

It renders the control's **children**, not the control: the runtime reconciles
a component's output *into* its host node, and the host is already the
`<radio>` element — authored, or built by [`Radio::view`]. Returning the whole
widget view would nest a second radio inside the first.

[`Radio::view`]: super::Radio::view
