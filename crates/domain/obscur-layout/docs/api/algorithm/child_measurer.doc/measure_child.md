Measures child `index` under `constraints`, returning its border-box size.

`index` refers to the position of the child within the `children` slice
passed to the algorithm. The returned size lies within `constraints` when
the implementation honours them; algorithms treat it as authoritative and
do not re-clamp it.
