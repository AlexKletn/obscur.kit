Matches `element` against all `sheets` in `environment` and returns its
cascaded values.

Identical to [`match_element`](Self::match_element) but each rule that was
authored inside an `@media`/`@container` group only contributes when its
[`RuleCondition`](crate::matching::RuleCondition)
[`holds`](crate::matching::RuleCondition::holds) against `environment`;
unconditional rules always contribute. A rule whose condition fails still
advances the shared `source_order` counter (exactly as a rule whose
selector list does not match does), so document-order tie-breaks are
unaffected by which rules the environment gated out.
