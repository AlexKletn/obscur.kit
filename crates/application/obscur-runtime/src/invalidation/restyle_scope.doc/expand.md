Records in `out` every node whose computed style can depend on `root`'s
interaction state, returning whether the scope could be bounded at all.

Returns `false` when the tree contradicts itself — `root`'s logical parent
does not resolve, or does not list `root` among its children — because the
following-sibling half of the reach cannot then be enumerated. A `false`
answer means "no bound was proven"; the caller must restyle everything
rather than trust a partial `out`.

A `root` that is not in the tree marks nothing and returns `true`: no
element can navigate to a node no child list mentions, so its state reaches
nobody.
