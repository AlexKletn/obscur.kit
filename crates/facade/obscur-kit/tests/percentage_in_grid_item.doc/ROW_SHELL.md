The same shell with a **definite** row track, which is what makes the block
axis measurable at all: a 240 viewport less 24px of padding top and bottom is a
192 content height, and `grid-template-rows: 100px` carves a 100-tall grid area
out of it. The two numbers are deliberately different, so a child resolving
against the wrong one cannot accidentally read correct.
