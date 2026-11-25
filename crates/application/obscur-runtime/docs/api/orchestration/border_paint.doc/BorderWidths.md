The four computed border widths of one node, in CSS shorthand order: top,
right, bottom, left.

They travel as a value rather than being re-read per shape because the paint
walk tries three border shapes in sequence — [`uniform_border`], then
[`uniform_dashed_border`], then [`border_fills`] — and every one of them needs
the same four widths. Reading them once at the top of the node's turn and
passing them down replaces twelve property lookups on a node with no border
(the panel and the rows) with four.
