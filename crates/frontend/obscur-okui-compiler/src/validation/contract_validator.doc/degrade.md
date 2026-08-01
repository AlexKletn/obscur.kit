Weakens `arity` to `inner`, keeping the weaker of the two.

Nesting only ever loosens the bound: a member already inside a `<For>` stays
unbounded when an `<If>` wraps it.
