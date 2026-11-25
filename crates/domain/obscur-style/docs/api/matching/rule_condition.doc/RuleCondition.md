An evaluable `@media`/`@container` condition tree.

Built by the CSS frontend from a parsed query and evaluated by the matcher.
The frontend maps a *media* feature it cannot represent (a boolean feature, a
non-`px` length, an unknown feature name) to [`Always`](Self::Always) so that
such rules are never silently dropped; the same unrepresentable feature in a
*container* query maps to [`Never`](Self::Never) instead, because no element
can currently be a query container (see [`FeatureQuery::holds`]).
