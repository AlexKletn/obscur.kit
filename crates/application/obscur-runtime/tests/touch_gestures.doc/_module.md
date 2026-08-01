Touch gestures end to end: several contacts at once, the long press, pointer
capture, and panning with its inertia (spec §2.4, §6.16).

Thirty-four tests that were unit tests inside `ui_runtime.rs` and are integration
tests here, unchanged. They could move because every one of them drives the
runtime through its **public** API — press, move, lift, tick, read the scroll
offset — which is also why they are the right proof that the harness in
`obscur-testkit` is usable from outside the crate: an integration test can reach
nothing else.

What they have in common is a *contact*, not a subsystem. A finger that presses
and waits is a long press; one that presses and travels is a pan; one that
travels fast and lifts is a fling; one that is taken away mid-gesture is a
cancel — and the interesting assertions are almost all about what a gesture must
**not** also do. Flicking a list must not press the row it started on. Holding
something and letting go must do one thing, not two. Catching a coasting list
must not open whatever was sliding past. Each of those is one gesture being
mistaken for another, which is why they are read together.
