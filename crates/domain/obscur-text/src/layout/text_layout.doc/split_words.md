Segments a run's glyphs into [`Word`]s at the line-break opportunities of
`text`. A break at byte offset `o` starts a new word at the glyph whose
cluster equals `o`; a hard break marks the preceding word for a mandatory
line break.
