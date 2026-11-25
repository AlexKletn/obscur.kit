Answers "which installed face can draw this character?" through fontconfig.

This is the seam that stops a character outside the application's own font
chain from rendering as nothing at all. The chain is the families an author
names plus the three generics a host installs; between them they cover Latin,
Cyrillic and Greek, so a CJK file name, an emoji, or a `⌫` on a key resolves
to no face and paints no glyphs — a blank, not a box.

Consulted only when every family in the chain has already declined a run, and
the answer is cached by the caller, so the cost is one `fc-match` per new
character rather than per run or per frame.

# What it does not do

It reads the face fontconfig names and nothing else: no `.ttc` collections
(the loader takes `.ttf`/`.otf` only, so a host whose CJK font is a
collection still draws nothing), and no attribute matching — a fallback face
is taken at its default weight and slant, because the question asked was
coverage, not style.
