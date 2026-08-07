Lowers one media query: its media type plus its (conjunctive) features,
negated when the query carries the `not` qualifier.

A feature the pipeline cannot represent sinks the whole query to
[`RuleCondition::Never`] and is reported into `unsupported`. The `not`
qualifier is deliberately *not* applied in that case: `Not(Never)` is
`Always`, which would restate the same ignorance as a claim that the query
holds. A `print` media type is different — that is a known answer, so
`not print` may negate it.
