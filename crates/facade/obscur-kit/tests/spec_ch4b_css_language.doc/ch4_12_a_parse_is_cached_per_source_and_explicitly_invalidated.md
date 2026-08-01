§4.12: "CSS parsing is cached per unique source/version and invalidated
explicitly."

The byte-identical source `S` is applied to scope A and then to scope B. The
second apply must be a **cache hit** — `S` is parsed exactly once across the
two applies and both scopes share the same parsed-sheet handle. After the
explicit invalidation call, the third apply is a miss and reparses.
