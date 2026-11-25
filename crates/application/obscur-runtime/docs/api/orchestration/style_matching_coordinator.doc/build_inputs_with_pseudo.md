Matches `sheets` against every element node in `tree`, sourcing each
node's interaction state from `pseudo`, and returns the resulting
[`StyleInputs`].

This is the pseudo-state-aware counterpart of
[`build_inputs`](Self::build_inputs): each element is wrapped in a
[`TreeElement::with_pseudo`] cursor so the matcher can honour
interaction-driven pseudo-classes (`:hover`, `:focus`, `:active`, …)
against the state the runtime recorded in `pseudo`. Passing an empty
[`PseudoStates`] is exactly equivalent to
[`build_inputs`](Self::build_inputs). `sheets` ordering and the
non-empty-only recording rule are identical to
[`build_inputs`](Self::build_inputs).
