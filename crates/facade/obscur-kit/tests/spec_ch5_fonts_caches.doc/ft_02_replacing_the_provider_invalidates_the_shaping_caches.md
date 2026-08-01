§5.18 "The provider is replaceable." With §5.19 "- glyphs;" and
"- correctness transparency;".

The failure this pins is a real one the suite already suffered: a font stack
can be swapped out and the tests keep passing because shaped runs are served
from caches keyed on text and size but **not** on the provider identity.
