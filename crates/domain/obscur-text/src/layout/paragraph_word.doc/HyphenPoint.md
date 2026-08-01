One interior hyphenation opportunity inside a [`ParagraphWord`]: where the
head ends, where the tail resumes, the head's advance width, and the source
byte a rendered hyphen glyph maps back to.

A break splits the word into `glyphs[0..head_end]` (the head, which may gain
a trailing hyphen glyph) and `glyphs[tail_start..]` (the tail). For an
algorithmic (interior) break `tail_start == head_end`; for an author soft
hyphen (`U+00AD`) `tail_start == head_end + 1`, so the invisible soft-hyphen
glyph is dropped at the break and replaced by the rendered hyphen.
