The typed classification of a change, ordered from shallow to deep.

A deeper kind implies all the (shallower) work of the kinds above it:
changing a node's tree structure ([`Tree`](InvalidationKind::Tree)) implies
re-styling, re-laying-out, re-painting and re-composing, whereas a
composition-only change ([`CompositionOnly`](InvalidationKind::CompositionOnly))
only requires the scene to be re-composed. The predicate methods
([`needs_restyle`](InvalidationKind::needs_restyle),
[`needs_relayout`](InvalidationKind::needs_relayout),
[`needs_repaint`](InvalidationKind::needs_repaint),
[`needs_recompose`](InvalidationKind::needs_recompose)) answer what work a
classified change requires.

The [`Ord`] instance orders variants by depth (shallow < deep), matching the
declaration order below.
