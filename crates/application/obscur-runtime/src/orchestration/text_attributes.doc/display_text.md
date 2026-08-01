The string a text node **shapes** on a frame, unifying the layout
(measure/arrange) and scene (compose) passes so both size and paint the
*same* characters (CSS Text 3 §4.1.1).

When `active` is the [`EditingView`] of the focused editable whose text this
node carries, the **live session buffer** is shaped RAW — `text-transform`
(§2.1) and `white-space` collapsing (§4.1.1) are deliberately **skipped** so
the caret's byte offsets stay exact — with any IME preedit spliced in at its
anchor for DISPLAY only (the buffer is untouched, spec §5.12). Otherwise the
retained `content` flows through the normal pre-shape pipeline: casing first
(§2.1), then whitespace processing (§4.1.1).

Passing the identical `active` view to **both** passes is what guarantees the
measured box height (line count) equals the painted line count while editing a
multi-line / collapsible field: the retained tree lags the session by a
keystroke, so a pass reading the retained value would size the box for the old
text while the other painted the new. With no active session (`None`) this is
exactly [`process_text`] — byte-for-byte the non-editing behaviour.
