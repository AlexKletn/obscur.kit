The viewport-relative projection of `boxes`, or `boxes` itself when no
scroll container overflows.

`boxes` is returned **by reference** in the unscrolled case, which is the
reason the projection is cached rather than computed on demand: the
overwhelmingly common tree pays nothing for the accessor at all.
