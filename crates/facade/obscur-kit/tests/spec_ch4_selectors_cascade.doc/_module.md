**SPECIFICATION-DERIVED** selector, specificity and cascade cases for
obscur.kit §4.13 ("complex selectors", ":nth-*") and §4.14 (CSS cascade,
`!important`), plus `display: contents`.

Every expected value below was produced by an agent reading the specification
— obscur.kit §4 and the W3C selectors-4 / css-display-3 text it normatively
cites — with **no access to this repository's source**. A failure here is a
defect report about the engine, and must not be "corrected" by editing the
expected value.

Translation note: this workspace's view tree has no element *tag names* that a
CSS type selector could match, so where the deriver wrote `em`, `li`, `p`,
`div` or `strong` as the **matching** part of a selector, a class of the same
name is authored instead (`.em`, `.li`, …) and the node carries that class.
The specificity arithmetic each case turns on is preserved or made *harder*,
never easier — see each test. Type selectors that are only there to *not*
match (the `:not(em, strong#foo)` arguments) are left verbatim.
