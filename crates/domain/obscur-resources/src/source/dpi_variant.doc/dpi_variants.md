Returns the DPI-variant candidates for `path` at a device `scale`, ordered
**densest first** and always ending at the authored path itself (spec §5.16
DPI variants).

The naming rule is the conventional `@Nx` suffix inserted before the
extension: `logo.png` at scale `2.0` yields `logo@2x.png`, then `logo.png`.
A caller tries each candidate in order and takes the first that loads, so a
project that ships no `@2x` asset transparently gets the base one — the
ladder is a *preference*, not a requirement. The returned `f32` is the
density of that candidate, which is the value a loaded representation records
as its [`ResourceMetadata::scale`](crate::state::ResourceMetadata::scale).

The requested density is `ceil(scale)` capped at `@3x`, so a fractional
display scale rounds **up** and never under-resolves: `1.5` asks for `@2x`
before the base. A scale at or below `1.0` (and any non-finite scale) asks
only for the authored path, so the common case allocates a single candidate
and behaves exactly as it did before variants existed.

The suffix is inserted in the final path segment only, so a directory
containing a dot (`assets/v1.2/logo.png` → `assets/v1.2/logo@2x.png`) is not
mistaken for an extension. A path with no extension takes the suffix at the
end (`logo` → `logo@2x`).
