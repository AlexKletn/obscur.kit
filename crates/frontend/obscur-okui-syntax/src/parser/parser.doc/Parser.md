A recursive-descent parser for the `.okui` markup language (spec §2.9).

The parser turns source text into a lossless [`SyntaxTree`] paired with any
recoverable diagnostics ([`ParseResult`]). It is *total*: it never panics,
never hangs, and never drops a byte — `result.tree().text()` always equals
the input, malformed or not.

# Algorithm

[`Parser::parse`] runs in three phases:

1. **Lex.** [`Lexer::tokenize`] produces a lossless [`Token`] stream ending
   in an [`Eof`](TokenKind::Eof) marker.
2. **Walk.** A recursive-descent walk over the token slice (with a `usize`
   cursor) emits a flat [`ParseEvent`] stream —
   [`StartNode`](ParseEvent::StartNode) / [`Token`](ParseEvent::Token) /
   [`FinishNode`](ParseEvent::FinishNode) / [`Error`](ParseEvent::Error).
   *Every* token, including whitespace trivia and stray/erroneous tokens, is
   emitted into the currently open node, which is what keeps the tree
   lossless.
3. **Materialize.** [`build_tree`] folds the balanced event stream into a
   [`SyntaxNode`] tree using an explicit node stack, collecting the
   [`Error`](ParseEvent::Error) events into the diagnostics vector.

# Grammar

```text
document  := (element | expression | text)*                 -> Document
element   := '<' Ident attribute* ('/>' | '>' child* close) -> Element
close     := '<' '/' Ident '>'                              -> CloseTag
attribute := Ident (':' Ident)? ('=' value)?                -> Attribute
value     := String | Number | Ident | '{' expr '}'        -> AttributeValue
expression:= '{' expr_tokens '}'                            -> Expression
text      := (any run of non-`<`, non-`{` tokens)          -> TextNode
```

The open tag `<Name …>` becomes an [`OpenTag`](SyntaxKind::OpenTag) child
node and the close `</Name>` a [`CloseTag`](SyntaxKind::CloseTag) child
node; both live directly inside the element node alongside its children.

## Control-flow classification

The element node is labelled [`Conditional`](SyntaxKind::Conditional) when
its tag name is `If`, `ElseIf` or `Else`, and [`ForLoop`](SyntaxKind::ForLoop)
when it is `For` or `VirtualFor`; every other tag yields a plain
[`Element`](SyntaxKind::Element). This is only a hint — the compiler performs
the authoritative classification later.

## Error recovery

The parser never aborts. On an unexpected token it records a [`ParseError`],
wraps the offending token(s) in an [`ErrorNode`](SyntaxKind::ErrorNode) and
advances so the cursor always makes progress. A missing or mismatched close
tag yields an [`ExpectedClose`](ParseErrorKind::ExpectedClose) /
[`MismatchedCloseTag`](ParseErrorKind::MismatchedCloseTag) diagnostic but the
element node is still finished. Because every token is emitted regardless,
losslessness holds even for malformed input.
