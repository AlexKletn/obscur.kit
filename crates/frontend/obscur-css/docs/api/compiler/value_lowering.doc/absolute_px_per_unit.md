The number of CSS pixels in one unit of an absolute (physical) length unit,
or [`None`] if `unit` is not one.

css-values-4 §6.2 fixes these ratios *definitionally* — "the absolute length
units are fixed in relation to each other and anchored to some physical
measurement", with `px` as their canonical unit:

* `1cm = 96px/2.54`
* `1mm = 1/10th of 1cm`
* `1Q = 1/40th of 1cm`
* `1in = 2.54cm = 96px`
* `1pc = 1/6th of 1in`
* `1pt = 1/72nd of 1in`

Because the ratios are fixed, an absolute unit carries no context-dependent
base and there is nothing to defer: it lowers straight to [`Length::Px`],
which is also what §6.2's "px is their canonical unit" asks a computed value
to serialize as. Unit names are ASCII case-insensitive (§6.2, note), so `1Q`
and `1q` are the same length.
