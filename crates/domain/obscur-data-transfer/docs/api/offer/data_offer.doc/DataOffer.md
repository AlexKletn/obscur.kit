The data being offered across a transfer, described as multiple MIME
representations (spec §6.20 `DataOffer`).

A `DataOffer` is what a source publishes and a consumer inspects: it lists
the same logical content under one or more
[representations](DataRepresentation), letting the consumer pick the richest
format it understands (rich text over plain text, an image over its URI). The
representations may be eager or lazy; the offer itself only records the menu.
Order is preserved, so callers may treat earlier representations as
preferred.
