Computes statistics by folding over a [`DisplayList`], counting each
command category.

`command_count` is the total command count; `draw_count`, `layer_count`,
and `clip_count` count the drawing leaf commands
([`PaintCommand::DrawPath`], [`PaintCommand::DrawText`],
[`PaintCommand::DrawShadow`], [`PaintCommand::DrawImage`], and
[`PaintCommand::BackdropFilter`]),
[`PaintCommand::PushLayer`], and [`PaintCommand::PushClip`] respectively.
Pops are counted only towards `command_count`.
