A literal run of text placed into the view tree.

`Text` is one of the minimal built-in primitives (spec §2.6). It differs
from the element primitives ([`Container`](crate::Container),
[`Image`](crate::Image), …): text is *real structure* in the desired tree
(spec §4.16), so it lowers to the dedicated [`ViewNode::Text`] variant
rather than a [`ViewNode::Element`]. Because text is not a component
element, `Text` carries no `COMPONENT_ID` and emits no properties — its only
content is the string itself.
