`Card` declares two §2.12 slots and renders **two regions**, each holding the
outlet of one of them.

Two, not one, on purpose: an assertion that the `h1` landed *somewhere* under
Card passes for an implementation that ignores the name and dumps every child
into the first outlet it finds. The `body` region is what makes "reached
`header`" mean more than "reached Card".
