Lowering a parsed `@media`/`@container` condition into `obscur-style`'s
CSS-syntax-free [`RuleCondition`], so the runtime matcher can gate rules by a
live environment (§4.13).

Only length features on the width/height (viewport) or inline-size/block-size
(container) axes with a `px` reference are represented as comparisons. Every
other feature — a boolean feature, a non-`px` length, `prefers-color-scheme`,
`orientation`, an unknown name — is unevaluable here, and both families fail
closed to [`RuleCondition::Never`]: a condition nothing can answer must not be
answered "yes". `@media` additionally reports each such feature as an
[`UnsupportedCssFeature`](crate::error::UnsupportedCssFeature), so the failure
reaches the author instead of silently disabling their rules. (`@container`
does not need its own report: the parser already reports every `@container`
at-rule, since nothing can establish a query container — see
[`MediaEnvironment`](obscur_style::matching::MediaEnvironment).)

`and`/`or` connectors are not distinguished by the parser, so a query's
feature list is treated as a conjunction; a media query list is a
disjunction, and the `not` qualifier negates a query — except over an
unrepresentable feature, where negation would turn ignorance back into a
claim.
