The per-node interaction [`PseudoState`] the runtime owns and the selector
matcher reads, keyed by [`NodeId`].

Interaction-driven pseudo-classes (`:hover`, `:focus`, `:active`, …) can only
match when *something* tells the matcher which nodes currently carry those
states. `obscur-style` deliberately does not decide that — it only consumes a
[`PseudoState`] bitset per element (via
[`MatchElement::pseudo_state`](obscur_style::selector::MatchElement::pseudo_state)).
`PseudoStates` is that source: the runtime maintains it as input events are
dispatched, and [`TreeElement`](super::TreeElement) reads it during matching so
authored `:hover`/`:focus` rules finally take effect.

A node with no recorded state resolves to [`PseudoState::empty`], so a fresh
(or fully cleared) map means "no interaction state anywhere" — exactly the
pre-Phase-3 behaviour. The map is a plain owned value: cheap to clone and to
reset between interaction epochs.
