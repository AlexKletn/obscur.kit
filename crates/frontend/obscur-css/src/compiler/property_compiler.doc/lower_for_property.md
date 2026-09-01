Lowers a value list for a specific property, applying property-aware rules.

A closed-set layout-keyword property (`layout_keyword_mapper` — `display`,
`overflow-x`, `flex-direction`, `text-align` and their kin) resolves its
keyword to the *reserved* cross-crate [`obscur_identity::InternedSymbol`] from
[`layout_keyword`], so the runtime layout bridge can compare keyword
identities without sharing this crate's interner.

CSS-wide keywords (`inherit` / `initial` / `unset`) still take priority. A
word the mapper does not know is **no value at all**: the mapper is the whole
grammar of these properties, so an unrecognised word is a declaration that
matches no grammar, and CSS Syntax 3 §9 drops it. The caller reports the drop
as an [`crate::error::UnsupportedCssFeature`], which is the honest answer both
for a typo (`overflow-x: sideways`) and for a word CSS has but this renderer
does not model (`display: inline`) — in either case the value cannot be
carried, and saying so is better than carrying a symbol that means nothing.

That mattered because an invented symbol is not inert. `OverflowAxes` reads
every symbol that is not `visible` or `clip` as clipping, so `overflow-x:
sideways` used to clip the box. `custom_value::lower_custom_value` reaches
this same function, which is what makes `overflow: var(--o)` and the literal
spelling of the same typo answer identically; the rule lives here and only
here.

Every other property uses the generic [`lower_value`] path unchanged, where an
open-set keyword is interned normally.
