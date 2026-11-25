Builds the [`FontAttributes`] (weight + slant) for a text node from its
computed `font-weight` and `font-style` (§4.14, §5.10).

Reads [`PropertyId::FontWeight`] — a bare [`PropertyValue::Number`] on the CSS
`1..=1000` axis — into a [`FontWeight`] (clamped by [`FontWeight::new`]), and
[`PropertyId::FontStyle`] — a [`PropertyValue::Keyword`] backed by one of the
reserved [`layout_keyword`] handles — into a [`FontSlant`]
(`FONT_STYLE_ITALIC` → [`FontSlant::Italic`], `FONT_STYLE_OBLIQUE` →
[`FontSlant::Oblique`], everything else including an absent property →
[`FontSlant::Normal`]). When both properties are absent this returns
[`FontAttributes::default`] — byte-for-byte the behaviour before per-node
weight/style resolution.

This helper feeds **both** the layout measure/arrange path and the scene
compose path so the attributes a text box is measured with are the attributes
it is painted with.
