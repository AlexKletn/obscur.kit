The active drag state machine (spec §6.21 enter/leave/over/drop/cancel).

A session is created with [`start`](DragSession::start) from a
[`DragSource`] and driven by pointer events: [`move_to`](DragSession::move_to)
tracks the pointer, [`enter`](DragSession::enter) and
[`leave`](DragSession::leave) mark crossing into and out of a
[`DropTarget`], and [`drop`](DragSession::drop) or
[`cancel`](DragSession::cancel) terminate it. Throughout, the session holds
the current [`position`](DragSession::position), the target
[`over`](DragSession::over) the pointer (if any), the negotiated
[`effect`](DragSession::effect), and the [`phase`](DragSession::phase).

Once the session reaches a terminal phase (see
[`DragPhase::is_terminal`](super::DragPhase::is_terminal)) every mutating method is a no-op, so a committed
drop or cancel cannot be reopened. Any provisional layout transaction that
materialises placeholders or insertion slots while a drag hovers is the
runtime's concern and is not modeled here.
