The kind of a token produced by the [`Lexer`](crate::lexer::Lexer).

This is the flat alphabet of the `.okui` markup language (spec §2.8): the
single-character punctuation of tags and expressions, the lexical atoms
(identifiers, string/number literals, raw text), plus trivia and the
end-of-input marker. It is deliberately small; structure (elements,
attributes, control flow) is recovered later by the parser, not the lexer.
