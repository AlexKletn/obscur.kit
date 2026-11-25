Which raw pointer-device event names the committed tree binds anywhere.

The cost gate for `pointermove`. A move arrives at the pointer's report rate —
hundreds of times a second on a trackpad — and resolving a binding means walking
the hit node's ancestor chain, which allocates. Doing that per move for an
application that binds nothing is a real cost paid for nothing, so the runtime
asks this first: seven bits, recomputed once per structural frame, and the walk
is skipped rather than merely empty.

`wheel` shares the gate, and needs it for the same reason rather than a weaker
one. A mouse's notches are rare, but a trackpad's two-finger scroll arrives as
`MouseScrollDelta::PixelDelta` at very nearly the rate its moves do — and the
wheel path pays *more* per event than the move path, because it hit-tests before
it can resolve a binding at all. So `handle_scroll` asks for this bit before it
maps a point.

The two crossing names — `pointerenter` and `pointerleave` — share the gate.
They are cheaper to begin with, because a crossing is dispatched only when the
hover chain actually changed, and the transition that says whether it did is
computed on every pointer event either way. But a chain change is not rare:
sweeping across a list crosses two boundaries per row, so the same claim has to
hold for them, and it holds for the same reason.

It is deliberately tree-wide and not per-node. A per-node answer would need the
same ancestor walk it is meant to avoid; a tree-wide one is the same trade
`CommandDispatch::propagate` already makes with `HandlerRegistry::has_listeners`
— free until *something* asks, then honest work.
