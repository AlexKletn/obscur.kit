A single token produced by the [`Lexer`](crate::lexer::Lexer).

A token stores the exact source slice it was lexed from ([`Token::text`]),
so the whole stream is *lossless*: concatenating the `text` of every token
(excluding the empty [`TokenKind::Eof`] marker) reproduces the source
byte-for-byte.
