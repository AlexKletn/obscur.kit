What a structural frame's retained value means for the focused session — the
answer [`ControlledReseed::decide`] returns and the runtime acts on.

The three outcomes are exhaustive and mutually exclusive, which is the point:
exactly one of "leave it alone", "note that the tree caught up" and "the app
rewrote this field" is true of any frame, and reading them as one enum is what
makes the discriminator below reviewable.
