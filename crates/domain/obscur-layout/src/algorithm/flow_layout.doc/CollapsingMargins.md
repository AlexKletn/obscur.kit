A set of vertical margins that are *adjoining* and therefore collapse into a
single used margin (CSS 2.1 §8.3.1).

The set is summarised by two running extremes — the largest positive margin
and the most-negative margin seen — so that:

* two positive margins collapse to the **larger** (positives don't add),
* two negative margins collapse to the **most-negative** (negatives don't
  add), and
* a mixed set collapses to their **sum** (`max_positive + min_negative`).

This is the canonical `{max_positive, min_negative}` reduction; folding each
margin in with [`adjoin`](Self::adjoin) and reading [`solve`](Self::solve)
yields the used margin for the whole adjoining run regardless of order.
