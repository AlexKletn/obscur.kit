An insertion point within a UTF-8 text buffer (spec §5.11 editing engine).

A caret names the [`offset`](Self::offset) — a *byte* offset into the buffer,
not a char or grapheme index — at which text would be inserted, together with
a [`CaretAffinity`] that disambiguates the offset when it lands on a soft
line-wrap boundary. Callers are responsible for keeping `offset` on a UTF-8
character boundary; this type treats it as an opaque `usize` position.

Carets are totally ordered by `offset` first and then by `affinity`, so a
collection of carets (as in multi-caret editing) can be kept sorted by
document position.
