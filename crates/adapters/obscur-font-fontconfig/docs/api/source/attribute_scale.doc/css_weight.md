Maps a fontconfig `%{weight}` to a CSS [`FontWeight`].

Exact at every anchor and piecewise-linear between them — the same shape
fontconfig's own `FcWeightToOpenType` uses, so a face fontconfig calls
`REGULAR` (80) is CSS `400` rather than an unrecognisable `80`. Values off
either end clamp to the nearest anchor.
