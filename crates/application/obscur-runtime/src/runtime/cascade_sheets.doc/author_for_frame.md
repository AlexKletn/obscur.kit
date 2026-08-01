Resolves the author sheets this frame matches: `supplied` when it is
non-empty — which is also remembered — and the last remembered set when it
is empty.

Retention is what makes a frame that owns no styling safe to render. A
scroll, a hover or an animation re-rendered with `&[]` must re-present the
*same* styled tree; without this it would restyle to the UA default and
the author's layout would collapse for one frame. The first-ever `&[]`
retains nothing and stays genuinely unstyled, so a single-render caller
that never had sheets is unaffected.

The consequence, stated because it is easy to reach for and does not work:
an empty set never *clears* the retained one, so a host cannot turn its
styling off by supplying `&[]`. It supplies a sheet with no rules instead.
