The runtime environment a [`RuleCondition`](crate::matching::RuleCondition)
is evaluated against: the viewport and, optionally, the querying container's
size, all in logical pixels.

An axis left unset (`None`) is *unknown*, and the two axis families resolve
an unknown differently (see
[`FeatureQuery::holds`](crate::matching::FeatureQuery::holds)):

- **Viewport** axes are non-restrictive when unknown, so an
  [`unbounded`](Self::unbounded) environment leaves every `@media` rule
  applying — the opt-out for callers that supply no environment. The runtime
  supplies a concrete [`viewport`](Self::viewport) so `@media (max-width:
  1px)` and friends actually gate.
- **Container** axes never hold when unknown, so an `@container` rule
  applies only when a caller has actually supplied a container size via
  [`with_container`](Self::with_container).

# No production caller sets a container size

[`with_container`](Self::with_container) currently has **no production
caller**, so in the live runtime every container axis is unknown and every
`@container` rule is inert. This is deliberate and fails *closed*: a
container's size is a layout output, but style matching runs over the whole
tree strictly *before* layout, and there is no `container-type` property, so
nothing can establish a query container. Wiring `@container` for real needs a
layout-interleaved restyle pass that this pipeline does not have. The CSS
compiler records an `UnsupportedCssFeature` for `@container` on
`CompiledStylesheet::unsupported` — the same channel `@import`, `:visited`
and `float` use. That field has no production reader today: every consumer
takes `.matchable` and drops it, so an author learns nothing unless the host
reads it. The rule is recorded, not reported.
