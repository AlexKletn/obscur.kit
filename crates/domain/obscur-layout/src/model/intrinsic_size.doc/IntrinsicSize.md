The intrinsic sizes a node's content wants, independent of any external
constraints (spec §4.1).

* `min_content` is the smallest size the content can take without overflowing
  (e.g. the longest unbreakable word).
* `max_content` is the largest size the content wants when given unbounded
  space (e.g. a paragraph laid out on a single line).

Sizes are fractional logical pixels; no rounding is applied (spec §4.6).
