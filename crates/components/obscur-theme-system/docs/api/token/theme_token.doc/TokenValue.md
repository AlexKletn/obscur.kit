The kinds of value a [`ThemeToken`](super::ThemeToken) can hold (spec §2.6).

A token is a single named design value; this enum enumerates the concrete
value shapes the theme system understands — colors, lengths and unitless
numbers, free text, font-family names, and animation durations. Because the
payloads include [`Color`] and `f32`, `TokenValue` derives `PartialEq` but
deliberately not `Eq`.
