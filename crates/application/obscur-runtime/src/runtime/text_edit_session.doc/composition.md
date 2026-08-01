The active IME composition anchored in `buffer`, if the field is composing
(spec §5.12). While `Some(_)` and active a preedit is shown inline (spliced
only for display) but kept OUT of `buffer` until it commits; a commit
replaces the anchor span in `buffer` with the committed text as one edit
(one composition = one undo unit).
