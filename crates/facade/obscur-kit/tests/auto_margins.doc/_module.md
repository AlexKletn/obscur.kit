End-to-end: **`auto` margins** absorb leftover inline space, and the
**logical axis shorthands** reach layout (spec §4.2, §4.13, CSS 2.1 §10.3.3).

`margin: 0 auto` is the single most-written centring idiom in CSS, and it used
to do nothing here: the layout model stored margins as four `f32`s, so `auto`
was folded to `0` before layout ever saw it. It cannot be folded early —
`auto` means *"take what is left over"*, and the leftover is not known until
the box has been measured.

Everything below asserts a real arranged box. The container is 300px wide and
the box 100px, so a centred box sits at exactly 100 — a number that is wrong
for every other outcome (0 if `auto` were dropped, 200 if the edges were
swapped).
