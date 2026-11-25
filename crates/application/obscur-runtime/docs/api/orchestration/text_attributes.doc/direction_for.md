Resolves a text node's base [`TextDirection`] from its computed `direction`
(§4.14, §5.10).

Reads [`PropertyId::Direction`] — a [`PropertyValue::Keyword`] backed by one
of the reserved [`layout_keyword`] handles — returning
[`TextDirection::RightToLeft`] when it is `DIRECTION_RTL` and
[`TextDirection::LeftToRight`] otherwise (including `DIRECTION_LTR` and an
absent property). The itemizer uses this as the paragraph's base direction
and assigns bidi levels from it; the same value drives both the measure and
compose shaping passes so they agree.

The runtime sets the locale's direction as the cascade default and propagates
it through inheritance. Explicit CSS direction takes precedence. The runtime
also supplies the locale's language to paragraph shaping.
