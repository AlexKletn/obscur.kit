Where a pointer lands in a field's text, and what a press or a drag there
selects (spec §5.11).

A **borrow bundle** over the retained frame — tree, computed styles, arranged
boxes and the scene's paragraph orchestrator — plus the live session, held for
one pointer event. Rule 5 of this crate's decomposition keeps the *acting*
out: every method **reports** a selection and the runtime applies it, because
applying it means the readonly gate, the `on-input` dispatch and the frame
request, and rule 2 forbids this entity from holding any of them.

# Why the four questions are one entity

They are one question — *which bytes did the user point at?* — asked at four
granularities, and all four must answer in the same coordinate system or the
selection tears. Every one of them starts from
[`offset_at`](Self::offset_at), the inverse of the caret-render advance walk:
a click at the rendered caret x of offset `i` has to come back as `i`, or the
caret appears somewhere the user did not click. The word and line brackets
then widen that offset through the buffer's own navigation engine — the same
engine `Ctrl`+arrow moves by — so a double click and a word-wise keyboard
selection cannot disagree about where a word ends. And the drag extends from a
fixed anchor to a **freshly resolved** offset over the *anchored* field rather
than wherever the pointer now is, which is what lets a sweep run off the end
of the field and still select to the end of its text.

The bracketing methods take the session by `&mut` because the buffer's word
and line motion is only reachable by *applying* moves to it: they walk the
buffer's own selection to find the bracket and hand the resulting range back
for the caller to set properly. That scratch use is the reason this holds a
`&mut TextEditSession` where its neighbour [`CaretScroll`](super::CaretScroll)
holds nothing at all.
