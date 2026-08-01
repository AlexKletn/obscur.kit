The replaceable port for resolving font families to concrete faces (spec
§5.18).

This module defines [`FontProvider`], the hexagonal port through which the
pure text domain turns a requested [`FontFamily`] plus [`FontAttributes`]
into a concrete [`FontFace`], and supplies a fallback chain when a family
cannot be resolved (spec §5.18). Interface adapters implement it over real
font backends — system fonts on native platforms, a bundled fallback set for
WASM and isolated environments — while the provider stays fully
**replaceable**. The colocated [`MockFontProvider`] is a deterministic
reference implementation over an in-memory face registry, so the port is
testable without any real font files.
