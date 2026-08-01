The advance of the word's trailing run of *collapsible* whitespace glyphs
(CSS Text 3 §3/§7.1). A word owns the space that follows it (the soft break
sits before the next word), so this is the part of `width` that **hangs**
when the word ends a line: it is excluded from the fit test and zeroed out
of the line's used width. `0.0` when the word has no trailing collapsible
space (or the mode preserves spaces).
