The uncommitted preedit (composition) string an IME shows inline while the
user is still composing (spec §5.12).

A preedit is *not* committed text: it is a transient overlay rendered at the
composition's anchor and replaced or discarded when the composition ends. Per
§5.12 the preedit is kept strictly separate from the buffer's committed
content until [`Composition::commit`](super::Composition::commit) folds it in.

The [`caret`](Self::caret) is a *byte* offset **within** [`text`](Self::text)
(where the IME wants the insertion point drawn), clamped to the text length
and snapped down to a UTF-8 character boundary by every constructor path. The
optional [`highlight`](Self::highlight) is a byte range within `text` marking
the active/target clause the IME emphasizes.
