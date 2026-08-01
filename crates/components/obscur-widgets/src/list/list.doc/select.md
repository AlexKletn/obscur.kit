Selects the item at `index`, returning `true` on success. In
single-select mode this replaces the selection with just `index`; in
multi-select mode it adds `index` to the selection. Returns `false`
(leaving state untouched) when `index` is out of range.
