Expands a `grid-column` / `grid-row` shorthand into its `*-start` / `*-end`
longhands (spec §4.13).

The value is split on a `/` delimiter into a start edge and an optional end
edge; each side is a `<grid-line>` — an integer line index (`3`), `span N`, or
`auto`. With no `/`, the end edge is `auto` (the line-based single-value form;
named-ident copying is out of scope), so `grid-column: 2` is `2 / auto` and
`grid-column: span 2` is `span 2 / auto`. An unparseable edge degrades the whole
shorthand to a single [`CompiledDeclaration::Unsupported`] rather than dropping
it.
