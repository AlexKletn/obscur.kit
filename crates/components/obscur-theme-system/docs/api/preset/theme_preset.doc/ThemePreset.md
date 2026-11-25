A named bundle of design tokens realising a visual mode (spec §2.6).

A preset is the unit an application selects to theme its UI: it pairs a human
name and a [`ThemeMode`] with a set of [`ThemeToken`]s keyed by name. Tokens
are stored in a [`BTreeMap`] so iteration and equality are deterministic
regardless of insertion order. The built-in [`light`](Self::light),
[`dark`](Self::dark) and [`high_contrast`](Self::high_contrast) presets each
seed a small, sensible token set.

The token values carry [`Color`] and `f32` payloads, so this type derives
`Debug`, `Clone`, and `PartialEq` only.
