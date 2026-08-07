Cascades every declaration of `candidates` whose owning selector `select`
accepts (returning that selector's specificity).

This mirrors [`StyleMatcher::match_element_in`](obscur_style::matching::StyleMatcher::match_element_in)
over the pre-filtered rules: each declaration keeps the document-order number
the whole sheet set gave it, so tie-breaks are the ones an unfiltered walk
would have produced.
