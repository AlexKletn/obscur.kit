Lowering a parsed `@media`/`@container` condition into `obscur-style`'s
CSS-syntax-free [`RuleCondition`], so the runtime matcher can gate rules by a
live environment (§4.13).

Only length features on the width/height (viewport) or inline-size/block-size
(container) axes with a `px` reference are represented as comparisons. Every
other feature (a boolean feature, a non-`px` length, an unknown name) lowers
to a per-family fallback: [`RuleCondition::Always`] for `@media`, so the rule
is gated permissively rather than silently dropped, but
[`RuleCondition::Never`] for `@container`, which fails closed because nothing
can establish a query container — see
[`MediaEnvironment`](obscur_style::matching::MediaEnvironment).

`and`/`or` connectors are not distinguished by the parser, so a query's
feature list is treated as a conjunction; a media query list is a
disjunction, and the `not` qualifier negates a query. Container queries have
no `not` qualifier in the parsed grammar, so no negation can flip a
failed-closed container feature back to holding.
