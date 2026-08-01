Lowers a container feature (container inline/block size axes).

An unrepresentable container feature lowers to [`RuleCondition::Never`], not
`Always`: a container query is evaluated against a container that nothing in
this pipeline can establish, so a permissive fallback would resurrect the
always-matches bug through the back door (`@container (orientation: portrait)`
would apply everywhere).
