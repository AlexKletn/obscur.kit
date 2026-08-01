A 2D affine matrix `[a, b, c, d, e, f]` mapping `(x, y)` to
`(a·x + c·y + e, b·x + d·y + f)` — the `matrix(a, b, c, d, e, f)` layout the
style/geometry crates use. Column 0 is `(a, b)`, column 1 is `(c, d)`, and
`(e, f)` is the translation.
