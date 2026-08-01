Rejects an expression outside the pure-read subset §2.10 allows, as a
`syn::Error` spanned at the offending expression.

The rule itself lives in the compiler
([`check_expression`](::obscur_okui_compiler::validation::check_expression)) —
the same function a compiled `.okui` file is checked with. Two copies of a
restriction is one too many: they drift, and the looser one is the one that
decides. What this adds is the span, which is the whole reason the macro path
reports better than the file path can: the author's build stops on the
expression they wrote.
