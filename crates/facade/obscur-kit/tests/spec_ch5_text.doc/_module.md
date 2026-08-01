**SPECIFICATION-DERIVED** text-subsystem cases for obscur.kit §5.10 ("Text
subsystem") and §5.18 ("Fonts").

§5 fixes no font and no metric conventions, so **no** absolute text width,
advance, line height or caret x is derivable from it. Every case below is
therefore written as a *relation* between measured quantities — "the advance
equals the advance of this other string", "x(3) == x(4)", "exactly 2 line
boxes", "the run's face is FontB" — never as a pixel count. A test in this
area that asserts a concrete width is asserting the shaper's constants back at
itself.

Every expected value was produced by an agent reading obscur.kit §5 and the
css-fonts-4 / css-text-3 text it normatively cites, with **no access to this
repository's source**.
