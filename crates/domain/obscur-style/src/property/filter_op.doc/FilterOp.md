A single ordered `filter` function op, as authored in a stylesheet.

The `filter` property is a `<filter-function-list>` applied in author order.
This enum models the functions the pipeline can honor today, preserving that
order:

- [`FilterOp::DropShadow`] — a `drop-shadow(...)`: a `box-shadow` DROP subset
  (`spread == 0`, `inset == false`), painted behind the element like a drop
  `box-shadow` (spec §5.2). The coordinator emits it as a `DrawShadow`.
- [`FilterOp::Blur`] — a `blur(<length>)` content blur; the radius is resolved
  to logical pixels at lower time. The coordinator brackets the element's
  subtree with a `PushLayer(Blur)` … `PopLayer`, and the Vello renderer runs a
  real separable gaussian pass over the composited subtree.
- The **color-matrix** ops — [`Brightness`](FilterOp::Brightness),
  [`Contrast`](FilterOp::Contrast), [`Saturate`](FilterOp::Saturate),
  [`Grayscale`](FilterOp::Grayscale), [`Sepia`](FilterOp::Sepia),
  [`Invert`](FilterOp::Invert), [`HueRotate`](FilterOp::HueRotate),
  [`Opacity`](FilterOp::Opacity) — each an affine color transform (CSS Filter
  Effects §). Because every one is affine, the coordinator folds a whole
  color-op chain into a single 4×5 matrix applied in one owned wgpu pass over
  Vello's output (Vello 0.9 has no color filters).

`drop-shadow` (a spatial shadow) and `blur` (a spatial gaussian) are NOT color
ops and stay their own passes; a filter list may freely mix them.

Because [`BoxShadow`] and the numeric amounts carry `f32` data, this type
derives [`PartialEq`] but not [`Eq`]/[`Hash`].
