The two speculative gestures a `Down` may arm — a pan and a long press — each
resolved against the frame the contact landed on (spec §6.16).

A **borrow bundle** over the retained frame: tree, computed styles, arranged
boxes, live scroll offsets and the realized virtual-list windows. It decides;
it arms nothing, scrolls nothing and schedules nothing. The caller seats what
it reports into [`PointerSessions`], which owns the recognizers.

# Why the two are one entity

Because they are armed by the same event and they **coexist by design**: a
finger that goes down inside a scrollable list that also binds a long press
arms both, and whichever the finger goes on to do — travel or wait — rules the
other out. That mutual exclusion lives in the recognizers, but it is only ever
reachable if both are armed from the same contact, so "what did this contact
arm" is one question with two answers rather than two questions.

They also share the gate that makes arming cheap enough to be speculative:
**direct contacts only**. A mouse drag inside a list is a text selection or a
data drag and turning it into a scroll would break both — the wheel is the
mouse's scroll gesture and it already works — while a mouse has a second
button for what a hold reaches. A finger has neither, so dragging the content
*is* how it scrolls and waiting *is* how it reaches the second button.
