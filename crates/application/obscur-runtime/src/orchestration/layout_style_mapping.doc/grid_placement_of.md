Assembles a [`GridPlacement`] from the four computed grid-line longhands
(`grid-column-start` / `-end` / `grid-row-start` / `-end`).

Each edge is read via [`grid_line_of`]; an absent edge is [`GridLine::Auto`], so
a node that authored no placement yields the all-[`GridLine::Auto`] default —
byte-for-byte the historical auto-placement behaviour.
