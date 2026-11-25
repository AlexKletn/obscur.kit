What a text leaf measures to: [`measure_text_node`] and the single-glyph
probes it wraps with.

A [`NodeKind::Text`](obscur_tree::node::NodeKind::Text) node carries no CSS
width or height, so the measure pass sizes it itself (spec §5.10): shape the
content once through the multi-font [`ParagraphShaper`], fold CSS Text 3
letter-/word-spacing and tab stops into the advances, wrap greedily to the
offered width, and report the wrapped paragraph's size.

Its vocabulary is the shaping stack's — strut, segment metrics, hyphen glyph,
ellipsis glyph, tab-stop width — and its interface is one call from the
measure pass. The four single-glyph probes are here rather than beside their
callers because they all answer the same question ("what does *this one*
character advance, shaped exactly the way the compose pass will shape it?")
and they all share [`measure_face`], which is what keeps the measured box and
the painted glyphs font-consistent.
