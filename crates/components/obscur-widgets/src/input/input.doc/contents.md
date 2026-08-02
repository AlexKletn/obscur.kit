The field's body: the single [`Text`] node carrying the current value, which
[`view`](Self::view) installs under its own root and a mounted
[`InputComponent`] regenerates from its props.

It is an associated function on the value rather than a method because the
component has no `Input` — it has the decoded props of one node — and the two
callers must produce the identical fragment or mounting the built-in would
change what an existing program renders.

[`Text`]: obscur_view::view::ViewNode::text
[`InputComponent`]: super::InputComponent
