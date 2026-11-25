A node's positioning insets — the `top` / `right` / `bottom` / `left`
offsets used by a positioned node (spec §4.1; CSS `inset`).

Each edge is a [`Dimension`], so it distinguishes a definite offset
([`Dimension::Px`] / [`Dimension::Percent`]) from `auto`
([`Dimension::Auto`], the initial value). This is separate from the pixel
[`Insets`](obscur_geometry::Insets) used for margin/padding/border, which
cannot carry `auto` or percentages.

The insets take effect for a [`Position::Relative`](crate::model::Position::Relative),
out-of-flow, or [`Position::Sticky`](crate::model::Position::Sticky) node; a
[`Position::Static`](crate::model::Position::Static) node ignores them entirely.
