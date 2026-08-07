Whether two laid-out boxes are the same rectangle, to a hundredth of a pixel.

The tolerance is there because the two rectangles being compared arrive by
different routes — one is an element's used size, the other a percentage of it
— and neither an exact-equality test that could fail on a rounding bit nor a
loose one that would let a real `padding: 1px` through is what these tests
want. A hundredth of a pixel is far below anything a stylesheet can express.
