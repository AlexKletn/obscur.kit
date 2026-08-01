Applies an IME composition `event` against the focused editable's editing
session (spec §5.12 IME), returning whether it was applied.

A composition lets the user compose text (a *preedit*, shown inline but
**not** inserted into the buffer) then commit it — the standard path for
CJK and other complex input. The event maps 1:1 onto the session's
composition lifecycle:

* [`Start`](CompositionEvent::Start) anchors a fresh composition at the
  current caret/selection; the preedit begins empty.
* [`Update`](CompositionEvent::Update) stores a new preedit on the
  composition. The preedit is shown (spliced for display, underlined) but
  is **never** written into the buffer, so [`editing_text`](Self::editing_text)
  is unchanged while composing (spec §5.12 "preedit is separate from
  committed text").
* [`Commit`](CompositionEvent::Commit) replaces the composition's anchor
  span in the buffer with the committed text as a **single** edit (one
  composition = one undo unit) and fires the field's
  [`EVENT_INPUT`](obscur_view::view::EVENT_INPUT) handler with the resulting
  text. Because the preedit was never inserted, an empty anchor makes this
  an insert at the caret.
* [`Cancel`](CompositionEvent::Cancel) drops the preedit with no buffer
  change.

Returns `false` (a no-op) when no editing session is active, or on an
out-of-order event (an `Update`/`Cancel`/`Commit` with no live composition).
Every applied event requests a [`FrameReason::Input`] frame so the preedit
(or the commit) reaches paint.
