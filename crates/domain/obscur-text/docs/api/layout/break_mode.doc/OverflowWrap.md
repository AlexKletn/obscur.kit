The computed `overflow-wrap` value (CSS Text 3 §5.2; the legacy alias
`word-wrap` maps onto it).

`overflow-wrap` permits a *last-resort* break inside an otherwise unbreakable
run of text, but **only** when the word would not otherwise fit on a line by
itself — i.e. to avoid content overflowing its line box. It never breaks a
word that already fits on its own line.
