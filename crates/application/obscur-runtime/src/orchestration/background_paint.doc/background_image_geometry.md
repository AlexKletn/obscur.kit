Resolves the CSS background placement geometry for a node's resolved image
into `(tile, repeat_x, repeat_y)` — the fields the
[`DrawImage`](obscur_scene::command::PaintCommand::DrawImage) command carries
(spec §5.2, CSS Backgrounds §3).

The `tile` is one copy of the image: its size comes from `background-size`
against the image's intrinsic pixel size and the paint `region` (the border
box), and its origin comes from `background-position` (the CSS position
formula). `repeat_x` / `repeat_y` are the `background-repeat` axis flags.
