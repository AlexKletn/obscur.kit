The lowering table from obscur's domain paint vocabulary onto `tiny-skia`'s
(spec §5.2, crate-arch §1.11 adapter boundary).

One question, asked of every value the display list carries: what is the
`tiny-skia` form of this domain geometry, path, transform, fill rule, colour or
paint? Every arm is total and side-effect free, and each answers `None` rather
than panicking where `tiny-skia` rejects a degenerate input — the renderer then
skips that mark, which is how a malformed display list stays panic-free.

This module is the *only* place a domain value crosses into `tiny-skia`, so the
hexagonal boundary is one file wide: everything here is `pub(crate)`, and no
`tiny_skia` type reaches a domain API through it.
