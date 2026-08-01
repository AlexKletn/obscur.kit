A bound **attribute** reaches the cascade, not just the tree: `class={theme}` is
substituted before reconcile, so the matcher sees the class the data holds and the
element is styled by it.
