A **floor** is what makes type stop following the window: below it the label
keeps its size while the box around it carries on shrinking, and ends up
several times wider than the key it sits in.

Measured on the demo's own shape, this was 18px of label inside a 1.5px key.
A ceiling alone — `min(<vmin>, <px>)` — keeps type sane on a wall-sized
window and lets it shrink the whole way on a squeezed one.
