Where a [`FontFace`](crate::font::FontFace)'s bytes come from: a
system-installed face, a bundled fallback face, or in-memory data (spec
§5.18).

The kit itself never reads system fonts: the platform
[`FontProvider`](crate::font::FontProvider) resolves [`FontSource::System`]
and [`FontSource::Bundled`] against the OS or a bundled fallback set, while
[`FontSource::Memory`] carries the font bytes directly for WASM and isolated
environments. Only `Memory` exposes bytes here.
