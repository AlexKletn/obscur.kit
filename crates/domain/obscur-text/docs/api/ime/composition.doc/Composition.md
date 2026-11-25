An active IME composition session anchored in the target buffer (spec §5.12).

A composition ties an [`anchor`](Self::anchor) — the byte range in the buffer
the composition occupies and will replace — to a live [`preedit`](Self::preedit)
and a [`state`](Self::state). The anchor is established at the primary caret
when the session [`start`](Self::start)s (multi-selection is restricted while
composing, so there is a single anchor); while
[`Composing`](CompositionState::Composing) the preedit is shown inline at that
range.

The typed lifecycle mirrors §5.12: [`start`](Self::start) →
[`update`](Self::update)\* → [`commit`](Self::commit) or
[`cancel`](Self::cancel). Per §5.12 "one composition is one undo unit"; that
undo grouping is enforced by the editing engine, which wraps the commit in a
single [`UndoKind::Composition`](crate::editing::UndoKind::Composition) unit.
This type only models the composition *state* — it performs no buffer edits;
the caller applies [`commit`](Self::commit)'s returned text as a replacement
over the anchor.
