Attribute names always permitted on any element whatever its contract's
declared prop surface: the structural keys, the `requires="cap"` capability
declaration (§2.13), `slot` — a child's slot assignment (§2.12), which is
addressed to its *parent*'s contract and so is never the child's own prop — and
`style`, which is styling addressed to the cascade rather than a prop the
component reads, exactly like `class`.
