A CSS 2D `transform` resolved to a single 2x3 affine matrix.

The authored `<transform-function>+` list (`translate`, `scale`, `rotate`,
`skew`, `matrix`) is composed **left-to-right** by the CSS lowering into one
affine, stored here as the six coefficients `[a, b, c, d, e, f]` of a
`matrix(a, b, c, d, e, f)` — the same layout
[`obscur_geometry::AffineTransform`] uses. A point `(x, y)` maps to
`(a·x + c·y + e, b·x + d·y + f)`.

`transform: none` lowers to the distinguished [`Transform::NONE`] — an identity
matrix flagged as *not authored* — while an authored `<transform-function>+`
list lowers to a [`Transform::new`] value, **even when that list resolves to the
identity matrix** (e.g. `translateX(0px)`). The distinction matters because
css-transforms-1 §3 makes "any value other than `none`" establish a containing
block for fixed/absolute descendants, regardless of the resolved matrix; see
[`Transform::establishes_containing_block`]. The scene coordinator wraps the
affine with its transform origin (`translate(origin) · matrix ·
translate(-origin)`) when it brackets the element's subtree.

Because the coefficients are `f32`, this derives [`PartialEq`] but not
[`Eq`]/[`Hash`].
