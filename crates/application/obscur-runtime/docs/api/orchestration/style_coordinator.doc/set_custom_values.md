Installs the grammar a custom property's declared text is parsed with.

The application tier cannot depend on a frontend crate (§1.11), so the CSS
grammar arrives here as a port rather than a call — the same route the
`style` attribute's compiler takes. Without it, `--font-body: Inter,
sans-serif` reaches `font-family` as text nothing can read, and the property
computes to `unset`.
