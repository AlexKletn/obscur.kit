Lowers a value list for a specific property, applying property-aware rules.

The four closed-set layout-keyword properties (`display`, `flex-direction`,
`justify-content`, `align-items`) resolve their keyword to the *reserved*
cross-crate [`obscur_identity::InternedSymbol`] from [`layout_keyword`] so
the runtime layout bridge can compare keyword identities without sharing this
crate's interner.
CSS-wide keywords (`inherit` / `initial` / `unset`) still take priority, and
a keyword outside the property's closed set falls back to the ordinary intern
path so it round-trips rather than being dropped. Every other property uses
the generic [`lower_value`] path unchanged.
