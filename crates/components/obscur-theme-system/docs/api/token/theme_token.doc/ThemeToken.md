A named design token pairing a stable name with its [`TokenValue`] (spec §2.6).

Tokens are the atomic, themeable values a preset is built from. The `name` is
a dotted, namespaced key such as `"color.primary"`, `"space.md"`,
`"radius.sm"`, or `"font.body"`; the `value` carries the concrete data. A
[`ThemePreset`](crate::preset::ThemePreset) is little more than an ordered
collection of these tokens keyed by name.

The [`Color`] and `f32` payloads block `Eq`, so this type derives `Debug`,
`Clone`, and `PartialEq` only.
