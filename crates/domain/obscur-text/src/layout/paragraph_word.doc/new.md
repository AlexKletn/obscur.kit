Builds a word from its `glyphs`, computing its advance width and its
interior hyphenation points from `offsets` (the hyphenator's algorithmic
breaks and the author's `U+00AD` breaks).

An interior point sits **before** a glyph whose cluster is an algorithmic
offset (the head runs up to it, the tail resumes at it). A soft-hyphen
point sits **at** a `U+00AD` glyph: the head ends before it and the tail
resumes after it, dropping the zero-advance soft-hyphen glyph. Neither
point is emitted at the word's first glyph (an empty head) or where it
would leave an empty tail. Points are sorted ascending by head width.
