Builds an empty [`FontRegistry`](crate::text_harfrust::FontRegistry) — the
real [`FontResolver`](crate::text::font::FontResolver) that picks a face
from a fallback chain by true `cmap` coverage.

Register faces on it (each `register` returns the
[`FontId`](crate::text::font::FontId) assigned in order `0, 1, …`), then
install it with
[`set_font_resolver`](crate::runtime::UiRuntime::set_font_resolver). Use
the **same** id↔bytes assignment on the renderer's `register_font` so
scene run handles line up with the renderer's fonts, and hand its
[`face_metrics`](crate::text_harfrust::FontRegistry::face_metrics) to
[`Self::text_shaper_without_font`] so the faces also size their own line
boxes.
