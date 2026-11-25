The runtime-matchable stylesheet: every style rule lowered into the
`obscur-style` selector + [`BlockDeclaration`] model the
[`StyleMatcher`](obscur_style::matching::StyleMatcher) consumes (§4.13).

This is an *additional* output alongside [`declarations`](Self::declarations)
(which pre-resolve the cascade context per selector): here each rule keeps
its authored selector list and block, and the matcher supplies specificity,
origin and source order at match time. Only standard longhand declarations
enter a block; custom properties and unsupported declarations are recorded
on the other fields, not here. Style rules nested inside `@media` /
`@container` groups are lowered with their group's condition **attached**
as an `obscur-style`
[`RuleCondition`](obscur_style::matching::RuleCondition), so the matcher
only lets them contribute when that condition holds against the runtime
environment (§4.13); the condition is *also* modelled, alongside the
group's flat declarations, in [`conditional_groups`](Self::conditional_groups).
