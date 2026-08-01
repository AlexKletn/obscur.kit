Lowers one component of a box shorthand to a [`PropertyValue::Length`].

Accepts the same lengths as the longhand path (`px`, `%`, `auto`) plus a
unitless `0` (treated as `0px`); anything else yields [`None`].
