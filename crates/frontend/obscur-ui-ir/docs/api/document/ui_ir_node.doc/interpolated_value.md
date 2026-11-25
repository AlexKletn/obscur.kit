Returns the interpolated expression when this node is an
[`interpolation`](Self::interpolation), else `None`.

An authored `<Expression/>` is a [`Component`](UiIrNodeKind::Component) —
both front ends classify an uppercase tag that way — so it never answers
here, and no document can spell its way into being an interpolation.
