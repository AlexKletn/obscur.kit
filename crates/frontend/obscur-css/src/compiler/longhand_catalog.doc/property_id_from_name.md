Resolves a CSS property name to its [`PropertyId`], case-insensitively.

Matches against every modelled *longhand*'s canonical [`PropertyId::name`].
Returns [`None`] for any name outside that set — including the `margin` /
`padding` shorthands, which are expanded to longhands by
[`super::PropertyCompiler::compile_all`], and custom properties, which are
handled separately by their `--` prefix.
