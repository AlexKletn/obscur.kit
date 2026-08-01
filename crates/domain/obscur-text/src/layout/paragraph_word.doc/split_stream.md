Segments a glyph `stream` into [`ParagraphWord`]s at the line-break
opportunities of the source text. A break at byte offset `o` closes the
word before the first glyph whose cluster reaches `o`; a hard break marks
that word for a mandatory line break. Each word records its trailing
collapsible whitespace and its interior hyphenation points (both from
`offsets`), see [`new`](Self::new).

Breaks are consumed by offset (ascending), so a break offset that lands
**inside** a ligature or **past the last glyph** (the end-of-text `\n` of
`"ab\n"`, whose cluster no glyph carries) is not lost: a trailing hard break
still marks the final word mandatory (CSS Text 3 §4.1.1), and a hard break
with no preceding glyphs (a leading/consecutive `\n`) yields an empty
mandatory word — a forced empty line box.
