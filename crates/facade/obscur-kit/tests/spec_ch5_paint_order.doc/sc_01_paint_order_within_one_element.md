css-backgrounds-3 §6.1.3: "In terms of stacking contexts and the painting
order, the outer box-shadows of an element are drawn immediately below the
background of that element, and the inner shadows of an element are drawn
immediately above the background of that element (below the borders and border
image, if any)." §5.2 "Must match across backends: … - paint order;".

WEAKENED 2026-07-19: this asserted `ops.len() == 5` and then read the five
commands BY INDEX. The engine paints eight leaves, because it lowers the
four border sides to four separate fills rather than one border command — a
legitimate lowering choice, and the paint ORDER those eight are in is exactly
the order the quoted clause requires. Neither css-backgrounds-3 §6.1.3 nor
§5.2's "- paint order;" says anything about how many draw commands an
element decomposes into; only about their relative order. The case now
asserts the four ordering relations directly, which is both closer to the
clause and stronger against a reordering regression than positional indices
were — an implementation that painted the inset shadow above the border still
fails. Candidate spec amendment: if the number of leaves matters (it would,
for a Scene/Paint golden compared across backends — §5.2 lists
"- Scene/Paint snapshots;" under "Must match"), state whether border sides
are one command or four, because two conforming backends currently need not
agree.
