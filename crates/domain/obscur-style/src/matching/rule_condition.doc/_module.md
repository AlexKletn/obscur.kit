The evaluable `@media`/`@container` condition gating a
[`StyleRule`](crate::matching::StyleRule): [`RuleCondition`].

This is `obscur-style`'s own, CSS-syntax-free model of a conditional group's
condition. The CSS frontend lowers a parsed media/container query into one of
these, attaches it to the matchable rule, and the
[`StyleMatcher`](crate::matching::StyleMatcher) evaluates it against a
[`MediaEnvironment`] at match time — so a conditional rule contributes only
when its condition holds.
