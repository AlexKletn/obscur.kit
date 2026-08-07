`transform` and `transform-origin` lowering (CSS Transforms 1 §3, §6).

A transform list composes left-to-right into a single [`AffineTransform`], so
the entity is a *fold* rather than a list — with one exception the fold cannot
swallow: a `translate` written in a font- or viewport-relative unit. Those
distances are not knowable here, so the parsed functions DO reach the domain, as
a `TransformFunction` list that computed value folds once the length context
exists. The origin is the other half of the same reference frame, and its
component reader — a `<length-percentage>` or one of the six side keywords,
tagged with the axis it names — is the one piece the gradient parser borrows for
its own `at <position>`.
