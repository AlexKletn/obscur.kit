One drawing operation in a display list (spec §5.1).

A [`DisplayList`](crate::command::DisplayList) is a flat, ordered stream of
these commands. Each command is either a **leaf** ([`PaintCommand::DrawPath`])
or one half of a **push/pop pair** that brackets a nested region of the
stream:

- [`PaintCommand::PushClip`] / [`PaintCommand::PopClip`] bracket a region
  confined to a clip.
- [`PaintCommand::PushLayer`] / [`PaintCommand::PopLayer`] bracket a region
  composited into a layer.

Push/pop pairs must **balance**: every push has a matching pop, they nest
(last pushed is first popped), and the running nesting depth never goes
negative. A well-formed list is balanced; [`nesting_delta`] lets callers
check this by folding over the stream. [`PaintCommand::DrawPath`] is a leaf
and contributes no nesting.

[`nesting_delta`]: PaintCommand::nesting_delta
