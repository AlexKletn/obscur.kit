One authored style rule: a [`SelectorList`] paired with the block of
[`BlockDeclaration`]s it applies.

This mirrors a CSS qualified rule `selectors { block }`. The rule carries no
cascade context of its own; when an element matches the rule's selectors, the
[`StyleMatcher`](crate::matching::StyleMatcher) turns each authored
declaration in the block into a fully-contextualised
[`Declaration`](crate::cascade::Declaration).

A rule that was authored inside an `@media`/`@container` group additionally
carries the group's [`RuleCondition`]; the matcher only lets it contribute
when that condition [`holds`](RuleCondition::holds) against the runtime
[`MediaEnvironment`](crate::matching::MediaEnvironment). An unconditional
rule has no condition and always applies.
