Cascades every declaration whose owning selector `select` accepts (returning
that selector's specificity), over `sheets` in document order.

This mirrors [`StyleMatcher::match_element_in`](obscur_style::matching::StyleMatcher::match_element_in):
`source_order` advances over every encountered declaration so document-order
tie-breaks are preserved, but only accepted rules' declarations are collected.
