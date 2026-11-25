Validates that an `.okui` expression stays within the pure-read subset the
spec allows (spec §2.10).

A thin wrapper: it parses the attribute's text as a Rust expression and hands
it to [`check_expression`], which is the rule. The `okui!` macro applies the
same function to the `syn::Expr` it already has, so a document and an inline
invocation are held to one standard.

# It used to be a lexical scan

It looked for a bare `=` and a handful of keywords in the raw text, which is
all a validator can do without parsing — and it let `{load_config()}` through,
because nothing in that string is a forbidden *word*. The expression is real
Rust (that is what it lowers to), so it can simply be parsed; there was never a
reason for the file path to know less about an expression than the macro does.
