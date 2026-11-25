The main-axis space a margin edge finally **occupies**: what it reserved, or
its share of the line's free space when it is `auto` (CSS Flexbox §8.1).

The counterpart to [`reserved`], and the two disagree on exactly one input.
`reserved` answers "what has to come off the top before there is free space",
which for `auto` is nothing; this answers "where does the item actually sit",
which for `auto` is its share of what was left. Calling the wrong one is how
the free space gets counted twice.
