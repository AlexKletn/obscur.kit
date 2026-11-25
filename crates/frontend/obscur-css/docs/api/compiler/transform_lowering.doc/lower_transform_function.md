Lowers one `<transform-function>` into a [`TransformFunction`]; [`None`] for an
unknown name or malformed arguments.

Only the `translate` family can carry a length, so only it can come back
unresolved; every other function is a number at parse time and comes back as a
pre-multiplied matrix.
