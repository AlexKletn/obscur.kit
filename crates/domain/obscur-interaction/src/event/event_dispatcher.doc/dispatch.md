Walks [`path`](Self::path), advancing `event` through each step.

This is a pure simulation of routing *without* handlers — dispatching
handlers is the runtime's job. For each step, if the event's propagation
is already stopped, the walk halts: `stopped_at` records the previously
visited node, or, if the event was stopped before any step ran, the node
of the halting step. Otherwise the event's [`phase`](UiEvent::phase) is
advanced to the step's phase. Because there are no handlers, nothing is
consumed, so [`handled`](DispatchResult::handled) stays `false` and the
walk only stops when the caller pre-set propagation control.

The returned [`DispatchResult`] carries the event's final
default-prevented state.
