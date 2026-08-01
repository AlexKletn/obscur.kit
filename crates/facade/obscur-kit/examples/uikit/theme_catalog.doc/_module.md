[`themes`]: the `(theme_name, stylesheet)` table — the one item that knows
every authored stylesheet, plus the index vocabulary ([`MATERIAL_INDEX`],
[`THEME_NAMES`]) that the switcher bar and the host examples select through.

The table is the seam of the whole scene: adding a design system is one more
entry here plus one more sibling `*_theme.rs`, and nothing in the view moves.
