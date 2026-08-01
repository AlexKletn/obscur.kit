End-to-end verification that the **`Image` primitive's `src`** reaches paint
through the *real* live pipeline: an `Image::new("logo.png")` node carries its
source through reconciliation (as the retained node's `image_source`), the
runtime loads it with a sandbox-rooted `FsResourceLoader`, decodes it with the
`image`-crate `ImageDecoder`, caches it as a `SceneImage`, and the coordinator
paints it as a `DrawImage` over the element's box as the element's CONTENT,
sized by `object-fit` (spec §2.6, §5.15 load, §5.16 decode, §5.2 paint).

This closes the gap that the `Image` primitive built a node whose `src` never
became painted pixels: the primitive now RENDERS.
