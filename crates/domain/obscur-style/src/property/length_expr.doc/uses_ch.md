Returns `true` when any leaf anywhere in this expression is a `ch` term.

Exhaustive over the tree, because `ch` is not required to be at the top:
`clamp(10px, 20ch, 50%)` needs the `0` glyph measured just as much as a
bare `20ch` does. See [`Length::is_ch`].
