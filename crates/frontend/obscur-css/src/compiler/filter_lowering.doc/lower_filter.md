Lowers a `filter` value into a [`PropertyValue::Filter`] op list.

The `filter` property takes a space-separated `<filter-function-list>`. Two
functions are modelled, kept in author order:

- `drop-shadow(...)` → a [`FilterOp::DropShadow`] (via [`lower_one_drop_shadow`]),
  a `box-shadow` DROP subset painted behind the element.
- `blur(<length>)` → a [`FilterOp::Blur`] (via [`lower_one_blur`]), the radius
  resolved to logical pixels. A zero radius (`blur(0)`, `blur()`) yields **no**
  op, so it round-trips to the byte-identical no-blur path. A radius in a
  relative unit cannot resolve here at all (`rem` is the root font size), so the
  whole list rides out as a [`PropertyValue::FilterFunctions`] for computed
  value to fold.
- the color-matrix functions (`brightness`, `contrast`, `saturate`,
  `grayscale`, `sepia`, `invert`, `hue-rotate`, `opacity`) → the matching
  [`FilterOp`] color variant (via [`lower_one_color_filter`]), each an affine
  color transform the coordinator folds into one 4×5 matrix.

Any other (unrecognised) function is **silently dropped** — it has no
GPU-expressible path here, so faking it would be dishonest (documented).
`filter: none` and an empty value both lower to an **empty** list (painting
nothing). Returns [`None`] (degrading to an unsupported-feature diagnostic)
when a recognised function is present but malformed, so the author sees the
mistake rather than a silent no-op.
