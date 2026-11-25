Expands a `border-color` shorthand into the four `border-*-color` longhands,
using the same 1/2/3/4-value edge replication as `margin` / `padding`.

Each component is a single `<color>` (hex, `rgb()`/`rgba()`, or a named
color); an unparseable component, or an empty or over-long list, degrades the
whole shorthand to a single [`CompiledDeclaration::Unsupported`].
