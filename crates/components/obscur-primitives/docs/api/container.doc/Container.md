A grouping box that lays out and clips a set of child nodes.

`Container` is one of the minimal built-in primitives (spec §2.6): the
terminal view-element builders that widgets and apps compose views from. It
carries no visual or layout style of its own — placement, sizing, and
clipping come from the style/CSS layer keyed on its stable
[`COMPONENT_ID`](Container::COMPONENT_ID) — so its only content is the
ordered children it groups.

Like every element primitive it is a plain config struct that lowers to a
[`ViewNode::Element`] via [`into_view_node`](Container::into_view_node); it
is a leaf builder, not a stateful component, so it does not implement the
component contract.
