css-align-3 §8.1: "The value normal represents a used value of 1em on
multi-column containers, and a used value of 0px in all other contexts."

Caveat recorded honestly: `gap` has no `normal` keyword in this workspace's
property model, so a parse failure would also yield 0 and pass this test for
the wrong reason. The `font-size: 32px` is retained from the deriver's setup
precisely because it makes the *wrong* answer (132) far from the right one.
