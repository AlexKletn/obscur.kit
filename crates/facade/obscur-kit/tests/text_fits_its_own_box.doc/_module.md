Text that fits the box layout gave it must not be broken by paint.

Layout and paint measure the same glyph advances and must agree. They did not:
layout summed the run once, paint rebuilt the sum word by word, and `f32`
addition is not associative — so a sentence measured at `293.9999` was re-wrapped
against `293.9999` and reported `294.00003`, putting its last word on a second
line (audit 2-D).

The consequence was not confined to the element: a flex row containing such a
label believed it was short of space and shrank its neighbour, so a badge beside
a wrapped note drew as `PRIVAC`.
