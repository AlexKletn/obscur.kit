Whether the list is balanced: every [`PaintCommand::PushClip`] /
[`PaintCommand::PushLayer`] has a matching pop, pushes and pops nest
correctly, and the running nesting depth never drops below zero.

Computed by folding [`PaintCommand::nesting_delta`] over the stream: the
running depth must never go negative (a stray pop) and must end at zero
(no dangling push). A well-formed list — such as one produced by a
balanced [`DisplayListBuilder`](crate::command::DisplayListBuilder) — is
balanced.
