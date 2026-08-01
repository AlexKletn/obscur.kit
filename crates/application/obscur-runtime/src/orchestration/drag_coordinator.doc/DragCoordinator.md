Drives a [`DragSession`] from pointer input and gates its effect behind a
[`ProvisionalTransaction`] (spec §3.2 provisional commit/cancel, §6.21 drag
and drop).

The [`DragSession`] state machine already knows the lifecycle; what it
deliberately leaves out — *"any provisional layout transaction that
materialises placeholders or insertion slots while a drag hovers is the
runtime's concern"* — is this type.

# What is provisional, and what is not

A drag's **live** state is not provisional: the pointer position and which
target is hovered change every frame and are simply read off the
[`session`](Self::session). Deferring those would mean a drag you cannot see.

What is provisional is the drop's **effect**. While the pointer is over an
accepting target, this coordinator stages the work that landing there would
cause — a [`Rerender`](UiOperation::Rerender) of that target, whose content
changes when it receives the payload. That staging is what
[`release`](Self::release) then either commits or throws away:

* a drop **commits** it — validated first, and applied as one transaction, so
  the frame does exactly the work the drop caused and no more;
* a cancel **discards** it — and because
  [`ProvisionalTransaction::cancel`] consumes the transaction, a cancelled
  drag cannot later commit. The guarantee is in the type, not in a comment.

A drag that wanders across five targets and is then abandoned therefore
applies nothing at all: each hover restages, and the final discard is free
because nothing was ever applied.

# Not carried

The drag **preview** ([`DragPreview`](obscur_data_transfer::drag::DragPreview))
is not painted — nothing here reaches the scene, so a host that wants a
pointer-following preview must draw it. Autoscroll
([`DragAutoscroll`](obscur_data_transfer::drag::DragAutoscroll)) is likewise
not driven. Both are scene/scroll concerns, and neither is what
commit/cancel is about.
