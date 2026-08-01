Free space on a flex line goes to the items that asked to grow (CSS Flexbox
§9.7 *Resolving Flexible Lengths*), and an item that did not ask stays at its
content size (§9.2.3 — an `auto` basis with no definite main size resolves to
the item's max-content size).

The defect these pin: a plain item with no flex properties absorbed the whole
line and starved the sibling that declared `flex: 1`. On an 800px row a text
item took 700px while a `flex: 1` sibling got **zero** — inverted, silently,
with no error anywhere.

It was invisible in the single-item case: `flex: 1` beside a fixed-width item
is correct, which is why the control below matters as much as the case.
