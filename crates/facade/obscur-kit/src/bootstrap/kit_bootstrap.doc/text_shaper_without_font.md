Builds a font-less real
[`HarfrustShaper`](crate::text_harfrust::HarfrustShaper) that shapes each
itemized run against the face bytes its
[`FontResolver`](crate::text::font::FontResolver) resolved
([`FontSource::Memory`](crate::text::font::FontSource::Memory)).

Pair it with [`Self::font_registry`] and [`Self::text_itemizer`] on a
runtime ([`set_text_shaper`](crate::runtime::UiRuntime::set_text_shaper),
[`set_font_resolver`](crate::runtime::UiRuntime::set_font_resolver),
[`set_text_itemizer`](crate::runtime::UiRuntime::set_text_itemizer)) to
wire the full multi-font fallback stack: itemize → resolve by coverage →
shape per item.

`faces` is that same registry's
[`face_metrics`](crate::text_harfrust::FontRegistry::face_metrics), taken
after every face is registered: a shaper is asked for line metrics by
[`FontId`](crate::text::font::FontId) alone, so without the table it would
shape real glyphs into line boxes sized by idealized ratios. Pass
[`FaceMetrics::empty`](crate::text_harfrust::FaceMetrics::empty) only when
there is deliberately no registry — line boxes are then ratio-sized, which
[`HarfrustShaper::idealized_line_metrics`](crate::text_harfrust::HarfrustShaper::idealized_line_metrics)
reports.
