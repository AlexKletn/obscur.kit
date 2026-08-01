A single pseudo-class in a compound selector (`:hover`, `:nth-child(2n+1)`).

Pseudo-classes fall into two families:

- **State** pseudo-classes (`:hover`, `:focus`, …) match against the
  interaction [`PseudoState`] bitset the element exposes. Each maps to the
  bit it requires via [`required_state`](Self::required_state).
- **Structural** pseudo-classes (`:first-child`, `:nth-child(…)`, `:root`,
  …) match against the element's position in the tree; they require no
  interaction state and so return [`None`] from
  [`required_state`](Self::required_state).

Any pseudo-class this crate does not model is preserved as
[`Other`](Self::Other) and matches nothing (a conservative, forward-safe
default), while still contributing class-level specificity.
