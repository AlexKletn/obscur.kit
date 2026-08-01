A node that renders its children at a named target elsewhere in the tree.

`Portal` is one of the minimal built-in primitives (spec §2.6). Its children
stay logically parented here (for data flow and events) but are visually
projected to a named portal target — the mechanism behind overlays, dialogs,
and tooltips that must escape their ancestor's clip or z-order. It is a leaf
element primitive: it lowers to a [`ViewNode::Element`] carrying the target
name as a property and its children as the element's fragment.

Unlike the childless text/image/foreign-surface primitives, `Portal` cannot
derive `Eq`: it holds child [`ViewNode`]s, which are only `PartialEq`
because their property values carry an `f64`.
