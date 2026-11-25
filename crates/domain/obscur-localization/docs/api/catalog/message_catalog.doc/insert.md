Parses `source` and registers it for `key` in `locale`, replacing any
previous pattern.

# Errors

Returns the [`MessagePatternError`] when `source` is not a valid pattern;
nothing is registered in that case.
