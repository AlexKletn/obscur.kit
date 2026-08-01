The parsed vector documents resolved from an element `src`, keyed like
[`image_cache`](Self::image_cache).

A document is parsed **once** and cached colour-independently: its
`currentColor` paints are still unresolved, so a theme change re-resolves at
compose time and never re-parses (spec §5.17).
