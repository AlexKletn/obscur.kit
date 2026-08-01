Lowers `decl` into one or more [`CompiledDeclaration`]s, expanding the
`margin` / `padding` box shorthands into their four longhands.

Every non-shorthand declaration lowers to exactly one result (the single
[`compile`](Self::compile) call, wrapped in a one-element `Vec`). A
`margin` / `padding` shorthand expands into up to four
[`CompiledDeclaration::Standard`] longhands (the CSS 1/2/3/4-value forms);
an unparseable shorthand value yields a single
[`CompiledDeclaration::Unsupported`]. This is the entry point the
stylesheet compiler uses so shorthands flow into both the flat cascade and
the matchable block.
