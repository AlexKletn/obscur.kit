Resolves a text node's line [`TextAlignment`] from its computed `text-align`
(§4.14, §5.10).

Reads [`PropertyId::TextAlign`] — a [`PropertyValue::Keyword`] backed by one of
the reserved [`layout_keyword`] handles — mapping each `TEXT_ALIGN_*` handle to
its [`TextAlignment`] variant. An absent property (or any unrecognised keyword)
resolves to [`TextAlignment::Start`], byte-for-byte the hardcoded behaviour
before per-node alignment resolution. The same value drives both the layout
measure/arrange pass and the scene compose pass so the sized box and the
painted lines agree.
