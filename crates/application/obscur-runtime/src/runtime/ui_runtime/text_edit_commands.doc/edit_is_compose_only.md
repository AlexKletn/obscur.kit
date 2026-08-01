Whether the just-applied edit can be flushed with a **compose-only** frame
([`FrameReason::TextEdit`]) because it provably cannot change any retained
box — a perf optimization that stays pixel-identical to the full-frame path.

A **non-mutating** edit (a caret move / selection change) never changes the
text, so no box can change: always compose-only.

A **mutating** edit is compose-only exactly when the focused field is
box-stable, which is [`ComposeOnlyEdit`]'s question: its documentation is
the written account of the three conditions and of why an unknown answers
conservatively. Any field that fails one of them takes a full `Input` frame
so its box relayouts — the measure↔compose box-sync invariant is preserved.
