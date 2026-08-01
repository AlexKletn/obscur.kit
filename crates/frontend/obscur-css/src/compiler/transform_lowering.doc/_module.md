`transform` and `transform-origin` lowering (CSS Transforms 1 §3, §6).

A transform list composes left-to-right into a single [`AffineTransform`], so
the entity is a *fold* rather than a list: the parsed functions never reach
the domain, only their product does. The origin is the other half of the same
reference frame, and its component reader — a `<length-percentage>` or one of
the six side keywords, tagged with the axis it names — is the one piece the
gradient parser borrows for its own `at <position>`.
