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

# Inert as of 2026-08-07

The paragraph above describes the **intent**. What actually happens today is
that this element lowers to an ordinary container: its children are laid out,
painted, clipped and stacked exactly where they were written, and
[`PROP_TARGET`](Self::PROP_TARGET) is read by nothing.

Nothing in the workspace maps a portal target *name* to a node, and
`obscur_tree::portal::Portal::apply` — the domain call that moves a node's
visual parent — has no production caller. `Dialog` used to wrap its surface in
`Portal::new("dialog-root")`; that wrapper was removed rather than left to
imply a projection that does not happen.

Keep using this type to *write* the intent — the §4.7 hit-chain oracle
(`spec_ch4b_hit_testing.rs`) is written against it — but do not expect a child
of it to escape an `overflow: hidden` ancestor. See `docs/api/12-reality.md`,
row *Portals*.
