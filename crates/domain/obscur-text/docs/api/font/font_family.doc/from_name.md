Parses a family from a name, mapping the well-known CSS generic keywords
(`"serif"`, `"sans-serif"`, `"monospace"`, `"cursive"`, `"fantasy"`,
`"system-ui"`) to [`FontFamily::Generic`] and everything else to
[`FontFamily::Named`] (spec §5.18).

Matching is case-sensitive against the canonical CSS spellings.
