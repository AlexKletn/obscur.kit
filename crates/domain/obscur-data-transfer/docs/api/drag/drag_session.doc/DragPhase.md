The lifecycle phase of a [`DragSession`](super::DragSession) (spec §6.21 enter/leave/over/drop/
cancel).

A session begins [`Started`](DragPhase::Started) — dragging but not yet over
any drop target — and moves to [`Over`](DragPhase::Over) while the pointer is
over an accepting target. It ends in one of two terminal phases:
[`Dropped`](DragPhase::Dropped) when a valid drop committed, or
[`Cancelled`](DragPhase::Cancelled) when the drag was abandoned or dropped
with no valid target.
