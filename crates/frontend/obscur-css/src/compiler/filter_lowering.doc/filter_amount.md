Parses one color-filter amount argument (`<number>` or `<percentage>`), where
`100%` is `1.0` and an empty argument list is the function's `default`.

Returns [`None`] (propagated as a malformed value) for more than one argument
or a non-number/percentage token.
