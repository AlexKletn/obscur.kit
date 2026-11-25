Resolves the `tab-size` tab-stop width in logical pixels for `content`
(CSS Text 3 §4.3), or `0.0` when `content` holds no preserved tab.

A `<number>` tab-size multiplies the advance of a space shaped through the
coordinator's default face — identical to the layout coordinator's — so both
passes resolve the same stop; a `<length>` needs no space. The space is only
shaped when a tab is present, keeping the tab-free path allocation-free.
