Resolves a [`SelectorTarget`] against a live runtime (spec §7.4, §7.5).

The descriptors were typed data that nothing consumed; this is the engine that
consumes them. Resolution is read-only and side-effect free — it answers
*which node*, never *what happens to it*; dispatch is
[`SyntheticDispatcher`](crate::input::SyntheticDispatcher)'s job.

# Why a coordinate does not resolve to a node here

[`Coordinate`](SelectorTarget::Coordinate) deliberately has **no** resolution
step. Hit-testing a point is the runtime's own job on the real input path, and
re-implementing it here would produce a second, subtly different answer — the
exact class of bug a headless test exists to catch. A coordinate is passed
through to the dispatcher as-is, which lets the runtime hit-test it exactly as
it would a real pointer.
