How a line that overflows its available width is handled (spec §5.10).

This is a *policy* — the layout records the intent, and a downstream painter
(an adapter) enforces it when rasterising. [`Visible`](Self::Visible) lets the
overflow spill past the edge, [`Clip`](Self::Clip) hard-cuts it at the
boundary, and [`Ellipsis`](Self::Ellipsis) replaces the trailing overflow with
an ellipsis. `Clip` and `Ellipsis` both truncate, which callers test with
[`is_truncating`](Self::is_truncating).
