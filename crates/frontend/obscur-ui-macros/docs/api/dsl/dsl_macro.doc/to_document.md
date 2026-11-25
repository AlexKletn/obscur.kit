Parses the DSL token stream into a [`UiIrDocument`] at expansion time.

The document is built, not emitted, so the caller can lower it further
while the macro is still running.

# Errors

Returns a [`syn::Error`] on empty input, on malformed element syntax, on
a mismatched closing tag, when a braced value is not a [`syn::Expr`], or
when an element breaks its component's contract (§2.12, §2.13).
