A single event in the parser's output stream (spec §2.9).

Rather than build the tree directly, the parser emits a linear sequence of
events which is later *materialized* into a lossless
[`SyntaxNode`](crate::syntax::SyntaxNode) tree (see
[`Parser::parse`](crate::parser::Parser::parse)). This decouples the
grammar walk from tree construction and mirrors the rowan/`rust-analyzer`
design.

## Well-formedness

A well-formed stream is *balanced*: every [`StartNode`](Self::StartNode) has
a matching later [`FinishNode`](Self::FinishNode), and [`Token`](Self::Token)
events only appear between a start and its finish. [`Error`](Self::Error)
events may appear anywhere and carry a diagnostic without affecting balance.
