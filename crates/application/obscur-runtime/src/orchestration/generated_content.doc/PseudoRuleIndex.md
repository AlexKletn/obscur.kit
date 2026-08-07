Which rules of a sheet set can produce generated decoration, and which cannot.

Built once per pass with [`PseudoRuleIndex::of`], before any element is looked
at. Almost no sheet authors `::before`, `::after` or `::part(placeholder)` at
all, and for such a sheet set all three lists come back empty — which is the
whole point: an empty list is a pass that never asks the selector engine a
single question, instead of one full match of every selector of every rule
against every element, three times over.

The index borrows the sheet set, and the sheet set is rebuilt every styled
frame, so it is rebuilt with it. That is deliberate: it costs one visit per
rule, against the deep sheet-set comparison that reusing an older one would
first have to pay for.
