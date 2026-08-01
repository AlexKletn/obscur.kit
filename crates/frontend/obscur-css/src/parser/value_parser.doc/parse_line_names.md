Parses a bracketed line-name group `[ a b … ]` after its opening `[` has been
consumed, up to and including the closing `]`.

Each name is a `<custom-ident>` (an `Ident` token); whitespace between names
is skipped. An unterminated group (EOF before `]`) or a non-ident token inside
the brackets is an error, so a malformed group degrades the declaration rather
than being silently accepted.
