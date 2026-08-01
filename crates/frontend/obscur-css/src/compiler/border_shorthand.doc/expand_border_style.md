Expands a `border-style` shorthand into the four `border-*-style` longhands,
with the same 1/2/3/4-value edge replication as `margin` / `padding`.

Each component is a `<line-style>` keyword folded to its reserved handle
(`none` / `solid` / `dashed`, via [`layout_keyword::border_style_from_name`]);
an
unrecognised keyword, or an empty or over-long list, degrades the whole
shorthand to a single [`CompiledDeclaration::Unsupported`].
