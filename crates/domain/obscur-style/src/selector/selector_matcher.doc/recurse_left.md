Walks the combinator immediately to the left of compound `k` (if any),
backtracking for descendant / subsequent-sibling combinators. `k == 0` is the
head, which has nothing to its left and always succeeds. Shared by
[`matches_at`] and
[`matches_complex_for_pseudo`](SelectorMatcher::matches_complex_for_pseudo)
so both agree on combinator semantics.
