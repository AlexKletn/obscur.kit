The physical property this logical one resolves into, or [`None`] when
`self` is not a logical property (spec §4.13, css-writing-modes-4 §6.2).

`rtl` is the element's computed `direction`; `wm` is its computed
`writing-mode`. Both axes rotate around `wm`:

* The **block** axis is vertical under `horizontal-tb` (`block-start` → top),
  and horizontal under the two vertical modes: `block-start` → the physical
  RIGHT under `vertical-rl` and the physical LEFT under `vertical-lr` (§6.2).
  `direction` never touches the block axis.
* The **inline** axis is horizontal under `horizontal-tb` and vertical under
  the two vertical modes, and it is `direction` that flips it: `inline-start`
  is the line-left side under `ltr` and the line-right side under `rtl`. In a
  vertical mode "line-left" is the physical top (inline text runs
  top-to-bottom), so `inline-start` → top under `ltr`, bottom under `rtl`.

Under the default `horizontal-tb` this is byte-for-byte the pre-`writing-mode`
mapping.
