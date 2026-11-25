Resolves a fixed-track [`Length`] to a concrete [`TrackSize`].

A `px` (or resolvable relative unit) becomes a [`TrackSize::Fixed`]; a bare
percentage becomes a [`TrackSize::Percent`] fraction (the `0..100` percent is
converted to `0..1`) carried symbolically to the layout algorithm, where the
container content extent — its basis — is known; `auto` (no numeric value)
degrades to [`TrackSize::Auto`].
