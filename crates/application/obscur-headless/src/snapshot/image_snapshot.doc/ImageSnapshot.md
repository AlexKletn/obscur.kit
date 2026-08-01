A deterministic, backend-neutral capture of a rendered raster image
(spec §7.4).

The rendered image is the one §7.4 stage that is *backend-dependent*: pixels
come out of a concrete rasterizer, not the backend-independent pipeline. This
type keeps `obscur-headless` free of any GPU or rasterizer dependency by
never *producing* pixels — [`from_rgba`](Self::from_rgba) takes whatever
raster a backend already made (for example the software `obscur-render-cpu`
backend's `tiny_skia::Pixmap::data`) and both hashes **and retains** it.

# Two comparisons, for two different questions

- [`PartialEq`] is **exact**: it compares dimensions and the FNV-1a content
  hash. This answers *"did the same backend produce the same bytes twice?"* —
  a determinism check, where any difference is a defect.
- [`compare_within`](Self::compare_within) is **tolerant** (spec §7.14). This
  answers *"does this render still match its golden, allowing for the
  variation §5.2 permits?"* — anti-aliasing, font rendering, backend
  precision.

The two are kept separate on purpose. A hash cannot express a tolerance (one
bit differs and the digest is unrelated), so the hash is retained for the
question it does answer, and the pixels are retained for the one it does not.

# Cost

[`from_rgba`](Self::from_rgba) retains the raster, so a snapshot is as large
as the image it captured. The buffer is shared behind an [`Arc`], so cloning
a snapshot — as [`SnapshotNormalizer`](super::SnapshotNormalizer) does — does
not copy it. [`from_parts`](Self::from_parts) builds a hash-only snapshot for
callers that want the stage identity without the bytes; such a snapshot
**cannot** take part in a tolerant comparison, and says so rather than
passing.
