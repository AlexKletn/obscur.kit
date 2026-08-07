The parsed vector documents resolved from an element `src`, keyed like
[`image_cache`](Self::image_cache).

A document is parsed **once** and cached colour-independently: its
`currentColor` paints are still unresolved, so a theme change re-resolves at
compose time and never re-parses (spec §5.17).

It is kept to what the tree still references:
[`retain_referenced`](Self::retain_referenced) sweeps it each composing frame,
so "parsed once" means once per appearance, not once per process. A parsed
document is small and its re-parse costs what its first parse cost, which is
why liveness decides this cache and a byte budget decides the image one.
