The pure measure pass: [`measure`].

Sizing without writing. `measure` answers "how big is this node under these
constraints?" and touches nothing but the [`MeasureCache`] — which is what
lets the arrange walk call it from *inside* an algorithm while holding a
mutable borrow of the results map. That borrow-cleanliness is the reason the
measure half is a separate entity from the arrange half rather than a branch
inside it.
