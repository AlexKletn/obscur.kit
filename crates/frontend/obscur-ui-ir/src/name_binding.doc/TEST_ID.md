Mirror of `obscur_view::view::TEST_ID` — a node's stable testing identifier
(spec §7.5).

Deliberately NOT a style attribute: it must not affect matching, specificity,
layout or visuals, so it is carried as an ordinary property and read only by a
selector resolver. It sits in its own `0x0000_02xx` namespace.
