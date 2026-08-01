The effect a completed drop will have on the dragged data (spec §6.21
effects).

A drag negotiates an effect between what the source permits and what the
target allows: dropping may [`Copy`](DragEffect::Copy) the data (leaving the
original), [`Move`](DragEffect::Move) it (consuming the original), or create
a [`Link`](DragEffect::Link) / reference to it. [`None`](DragEffect::None) is
the absence of any acceptable effect — a drop under `None` is a no-op that
cancels rather than commits. Use
[`first_allowed`](DragEffect::first_allowed) to negotiate the effect a drop
should carry from a target's allowed set and a preferred choice.
