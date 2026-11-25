Lifts an inner accumulator into `acc`: nested keyframes, conditional groups
and diagnostics rise to the enclosing scope, while the flat declarations and
transitions are wrapped in a [`CompiledConditionalGroup`] under `condition`.

The nested style rules also rise into the enclosing matchable set, but each
is now **gated** by `condition`: its `condition` is lowered into an
`obscur-style` [`RuleCondition`](obscur_style::matching::RuleCondition) and
attached to the rule, so the runtime matcher only lets it contribute when the
condition holds against the live environment (§4.13). A rule that already
carries a condition (it came from a group nested inside this one) is gated by
the conjunction of the outer and inner conditions.

Transitions, animation declarations and keyframe definitions carry the same conditions into runtime motion matching.
