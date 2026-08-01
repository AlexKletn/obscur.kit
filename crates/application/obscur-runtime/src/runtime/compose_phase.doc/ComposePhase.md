Step 5 of the frame (spec §3.4): compose the paint-ordered display list from
the styled, arranged tree, bracket it in the surface's root transform, and
install it as the scene root.

It is a **borrow bundle**, not a holder: every field is a borrow of state the
runtime owns across frames, taken for the duration of one phase and released
with it. The [`Scene`] it installs into outlives the frame; nothing else here
does.

# Why the phase is one entity rather than a run of statements

Its interface is one call that returns nothing — the frame's output is the
scene root it installs — and its internals are four *layers* stacked in an
order that is a rule rather than a habit:

1. the composed content itself, painted from the computed styles with the live
   editing view, the decoded images, the resolved scroll offsets and the §4.16
   generated content;
2. motion's presentation values (`opacity`/`transform`), layered onto a
   **throwaway** copy of the styles so they reach paint and stay unobservable
   through `computed_style` (spec §4.17);
3. §7.6's inspect highlight, appended *before* the root transform so it is
   traced in the same logical space as the boxes it outlines; and
4. the device-scale + content-scroll root transform, which is the only place
   logical px becomes physical px.

Every one of those layers is invisible from the outside and each is wrong if
it moves: a presentation value written onto the retained styles leaks a motion
sample into the cascade, and a highlight appended after the root transform is
drawn at the wrong scale.
