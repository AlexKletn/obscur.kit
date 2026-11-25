Resolving a computed CSS length into layout's own numeric vocabulary.

Everything the [`ComputedStyle`] → [`LayoutStyle`](obscur_layout::model::LayoutStyle)
bridge needs in order to turn a *length-valued* property into a
[`Dimension`], an [`Insets`] edge, a [`Margin`] or a bare pixel number: the
per-node [`LengthContext`] that gives `em`/`rem`/`vw`/`ch` their bases, and
the four readers built on it.

This is a separate entity from the keyword half of the mapping (see
[`layout_style_mapping`](super::layout_style_mapping)) because it has a second
consumer: `grid_templates` resolves a fixed track's relative units through the
same [`length_context`] / [`ch_width_for`] pair, and never reads a keyword.
