One child of a node, presented to a [`LayoutAlgorithm`] as identity + style.

[`LayoutAlgorithm`]: super::LayoutAlgorithm

Layout algorithms in this crate never walk the retained tree themselves — the
runtime does (spec §4.2). An algorithm receives the parent's style, the slice
of its children (each a `LayoutChild`), and a [`ChildMeasurer`] callback used
to size a child under chosen constraints. This type carries exactly what an
algorithm needs about a child *before* measuring it: which node it is and the
style that drives its participation (flex basis, grow/shrink, explicit sizes,
alignment overrides, ...).

[`ChildMeasurer`]: super::ChildMeasurer

All fields are `Copy`; the whole value is cheap to pass by value.
