Parses a BCP-47 `language[-script][-region]` tag, canonicalising each
subtag; returns `None` for anything outside that shape.

Rejected, deliberately: an empty string, a 1- or 4+-letter language, a
subtag that is not the right length *and* character class for its
position, and any tag with a fourth subtag. There is no partial success —
a caller that hands over `en-US-u-ca-buddhist` gets `None`, not a silent
`en-US`.
