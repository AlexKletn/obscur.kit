Vendor-prefixed properties, the way browsers have always handled them.

Two halves, and the second is what makes the first honest:

* `-obscur-` is this kit's own namespace, and a name in it that this compiler
  does not know is a FAULT — a typo in your own house.
* every other vendor's prefix is passed over in silence, because that is the
  entire contract of a prefix and the reason one stylesheet can serve renderers
  that do not know each other. It is not accepted as an alias for anything of
  ours: `-webkit-filter` does not become `filter` (spec §4.13 excludes vendor
  hacks, and this is the exclusion that matters).
