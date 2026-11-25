Tokenizer for the `.okui` markup language (spec §2.8).

The lexer is stateless: a single instance can tokenize any number of
sources. It is *total* — it never panics and never loses a byte. It scans
the source char-by-char (via [`str::char_indices`], so multi-byte UTF-8 is
handled correctly) tracking a running byte offset.

# Tokenization rules

The lexer runs in a single "markup" mode and recognizes, in priority order:

- the single-character punctuation `<`, `>`, `/`, `=`, `{`, `}`, `:` as
  [`LAngle`](TokenKind::LAngle) … [`Colon`](TokenKind::Colon);
- a double-quoted run `"..."` as [`String`](TokenKind::String), *including*
  the surrounding quotes. Escaping is minimal: a backslash escapes the next
  character, so a `\"` inside the literal does not terminate it. An
  unterminated string simply runs to end-of-input (kept lossless, no panic);
- an identifier `[A-Za-z_][A-Za-z0-9_.-]*` as [`Ident`](TokenKind::Ident);
- a number `[0-9]+(\.[0-9]+)?` as [`Number`](TokenKind::Number) (a trailing
  dot with no following digit is *not* consumed);
- a maximal run of whitespace as [`Whitespace`](TokenKind::Whitespace);
- any other maximal run of ordinary (non-special, non-control) characters as
  [`Text`](TokenKind::Text);
- a lone control character as [`Error`](TokenKind::Error).

A synthetic [`Eof`](TokenKind::Eof) token with empty text and
`offset == source.len()` is always appended.

## Losslessness invariant

Concatenating the [`Token::text`] of every produced token except the final
(empty) `Eof` reproduces the source byte-for-byte. This is verified by the
crate's tests.
