Reconciles the focused editing session with an **external** rewrite of the
field's retained value while it stays focused — the controlled-input reseed
(WHATWG HTML §4.10.5.5: a text control reflects its `value`, so when the app
rewrites that value the control displays the new value). Called after
reconcile on a structural frame, when the retained tree already carries the
app's value.

# The reseed discriminator (external rewrite vs the session's own edit)

The session is normally the **sole source of truth** for the focused field
(an *uncontrolled* input), and the compose/measure passes paint it — so the
retained tree must NOT clobber the buffer on every keystroke. Two conditions
gate a reseed, using the session's [`mirror`](TextEditSession::mirror)
watermark (the last retained value it reconciled against):

1. the retained editable value **changed** since the session last saw it
   (`retained != session.mirror()`). A field whose retained value never
   changes — a static, uncontrolled, one-shot-seeded field — never reseeds,
   so the user's in-flight typing is never fought (this is what keeps the
   measure↔compose "session leads the lagging tree" invariant intact); and
2. the new retained value **differs from the session's own live text**
   (`retained != session.text()`) — i.e. it is NOT just the session's own
   edit mirrored back through the app (a passthrough controlled input). A
   normalizing `on-input` (trim / uppercase / max-length) whose write-back
   differs from the buffer trips this and DOES reseed, so the displayed text
   tracks the model; a plain passthrough does not (it only advances the
   watermark).

On a reseed the caret is preserved (clamped into the new value); otherwise
only the watermark is advanced.
