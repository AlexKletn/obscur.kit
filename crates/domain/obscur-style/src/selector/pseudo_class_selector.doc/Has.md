`:has(S)` — the relational pseudo-class (selectors-4 §4.5).

Modelled for **specificity only**; the matcher answers `false` for it
unconditionally. Deciding it requires navigating *downward/forward* from
the subject, which [`MatchElement`](super::match_element::MatchElement)
deliberately cannot do — see the matcher's note and
`obscur-runtime`'s `RestyleScope`, whose invalidation bound is a proof
resting on that navigation surface. `obscur-css` reports the selector as
an unsupported feature so the rule is not a silent drop.
