An ordered, immutable sequence of [`PaintCommand`]s with a **deterministic
paint order** (spec §5.1).

The order of the commands *is* the paint order: a renderer executes them
front to back, so the same list renders identically across backends. The
list is a flat stream in which [`PaintCommand::PushClip`] /
[`PaintCommand::PushLayer`] open nested regions closed by their matching
pops.

A **well-formed** list is *balanced* — see [`is_balanced`]. Renderers rely
on this invariant; an unbalanced list has a dangling push or a stray pop and
cannot be consumed safely. Balance is checkable, not enforced: constructing
a `DisplayList` never validates it.

[`is_balanced`]: DisplayList::is_balanced
