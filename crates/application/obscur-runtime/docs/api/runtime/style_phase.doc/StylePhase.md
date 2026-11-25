Steps 2, 3, 3a and 3b of the frame (spec §3.4): assemble the cascade list,
drive the match + resolve through [`RestylePhase`], resolve §4.16's generated
content from the same sheets, record what the result was produced from, and
lay motion's sampled values over it before layout reads a single box.

It is a **borrow bundle**, not a holder: every field is a borrow of state the
runtime owns across frames, taken for the duration of one phase and released
with it.

# Why this is an entity beside [`RestylePhase`] rather than part of it

[`RestylePhase`] answers one question — *which nodes' styles does this frame
resolve, and to what* — and it answers it over the cascade tier alone. This
phase is what the **frame** owes around that answer, and every item of it is a
pairing rather than a step:

- the list that cascades is the UA sheet plus the retained author sheets, and
  only [`CascadeSheets`] may put the two together (§4.14);
- §4.16's generated content must be resolved from the *same* sheets and the
  *same* media environment the cascade used, or a `::before` is styled by rules
  the element it decorates never saw;
- the [`StyleKey`] recorded at the end is the sole warrant for the *next*
  frame's incremental restyle — a phase that resolved styles and failed to
  record their inputs leaves the next frame reusing a result it cannot justify;
  and
- the motion overlay (§4.17 style → motion → paint) has to land **after** the
  cascade and **before** layout, and the settled pre-overlay value has to be
  stashed as it goes, or the next frame's transition change-detection compares
  against a mid-animation sample.

Those four are invisible from outside and each is silently wrong if it moves,
which is exactly the "interface much narrower than its internals" that makes a
group an entity: the whole of it is one call returning this frame's styles.
