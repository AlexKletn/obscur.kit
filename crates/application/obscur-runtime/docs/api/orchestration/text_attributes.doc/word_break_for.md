Resolves a text node's [`WordBreak`] from its computed `word-break`
(CSS Text 3 §5.2).

Reads [`PropertyId::WordBreak`] — a [`PropertyValue::Keyword`] backed by a
`WORD_BREAK_*` reserved handle — mapping each handle to its variant. An absent
property (or any unrecognised keyword) resolves to [`WordBreak::Normal`], the
CSS initial.
