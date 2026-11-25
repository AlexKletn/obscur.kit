The layout input [`LayoutStyle`] derived for every node, keyed by [`NodeId`].

This is the runtime's bridge from `obscur-style`'s [`ComputedStyle`](obscur_style::computed::ComputedStyle) into
`obscur-layout`'s own vocabulary (spec §3.10, §4.2): layout consumes its own
[`LayoutStyle`], and the mapping from computed styles is the coordinator's
responsibility.
