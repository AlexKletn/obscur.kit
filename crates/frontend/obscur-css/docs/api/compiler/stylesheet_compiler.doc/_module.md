The compiler entry point: a parsed stylesheet into native structures (§4.12).

[`StylesheetCompiler`] walks a [`ParsedStylesheet`] and lowers it into a
[`CompiledStylesheet`] of `obscur-style` cascade declarations and custom
properties, `obscur-motion` keyframe tracks and transition templates, and the
conditional (`@media` / `@container`) groups that gate them — carrying
forward the parser's own errors and unsupported-feature diagnostics (§4.13).

Because `obscur_identity::InternedSymbol` is an opaque handle with no shared
global interner, the compiler owns its own [`CssSymbolInterner`]: every
keyword, custom-property name and animation name is interned here, and the
resulting table travels inside the [`CompiledStylesheet`] so consumers can
resolve a symbol back to its text.
