Returns the value of the first header whose name matches `name`
case-insensitively, or `None` (spec §5.15).

The match is ASCII-case-insensitive because HTTP header names are
case-insensitive; the first matching header wins.
