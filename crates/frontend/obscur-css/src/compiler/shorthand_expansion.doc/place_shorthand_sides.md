Returns the two longhand [`PropertyId`]s (`align`, `justify`) that a
`place-items` / `place-self` / `place-content` shorthand expands into, or
[`None`] for any other name. The CSS `place-*` grammar is `<align> <justify>`,
so the *align* side comes first (`place-content: <align-content>
<justify-content>`).
