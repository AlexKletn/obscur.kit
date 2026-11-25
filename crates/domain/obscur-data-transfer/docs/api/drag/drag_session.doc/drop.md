Commits a drop, returning whether it succeeded (spec §6.21).

Succeeds only when the session is in phase [`Over`](DragPhase::Over) with
a non-[`None`](DragEffect::None) effect: it enters phase
[`Dropped`](DragPhase::Dropped) and returns `true`. Otherwise — a drop
with no valid target — it enters phase [`Cancelled`](DragPhase::Cancelled)
and returns `false`.

A no-op returning `false` once the session is already terminal.
