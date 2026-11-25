Property name for the image source key.

Unlike the other primitive props (which live in the `0x00C1_NNXX` prop
space), the source points at [`obscur_view::view::SRC`] — the reserved
low-symbol attribute the reconciler extracts onto the retained node's
[`StyleAttributes`](obscur_view::view::StyleAttributes) — so an image's
`src` survives reconciliation as `image_source()` and the runtime can
resolve and paint it. Aliased here so `Image::PROP_SRC` still exists.
