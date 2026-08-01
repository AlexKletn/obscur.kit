Runs `tool` with `args`, returning its stdout on success.

Any failure — tool absent, non-zero status, non-UTF-8 output — collapses to
`None`. There is nothing a caller could do with the distinction that it
would not also do for "no fonts here".
