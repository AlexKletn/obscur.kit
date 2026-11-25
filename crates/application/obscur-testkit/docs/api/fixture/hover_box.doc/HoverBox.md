A 100×50 `.box`, and the two hover sheets that make hovering it cost either a
relayout or only a repaint.

The pair is the fixture. [`layout_hover_sheet`](Self::layout_hover_sheet) moves
`width` on `:hover`, so the layout phase *must* run;
[`paint_hover_sheet`](Self::paint_hover_sheet) moves `background-color`, a
paint-only property, so no box can have moved and the whole layout phase is
provably skippable (§3.4 p5, §4.3). Every assertion about incremental
invalidation is the difference between those two sheets over the same view.

[`row_view`](Self::row_view) is three siblings rather than one, so a hover on the
middle box has clean neighbours to *not* be about, and [`selectors`](Self::selectors)
is the bare `.box` selector list the CSS motion rules are keyed on.
