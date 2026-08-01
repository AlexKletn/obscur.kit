[`TokenCursor`]: reading an already-tokenized slice, and the span
conventions that go with it.

The cursor is the *reader* half of the lexical layer, and it is a separate
entity from the [scanner](super::css_tokenizer::Tokenizer) that produced the
slice: it holds no text, allocates nothing, and is constructed fresh by each
sub-parser over a borrowed `&[CssToken]`. [`super::stylesheet_parser`] does
not use it at all — the top-level rule reader walks its own owned `Vec` — so
this is the one piece of the parser module whose only callers are the sibling
sub-parsers ([`super::selector_parser`], [`super::value_parser`]).

[`slice_span`] lives here rather than with the token model because it shares
the cursor's end-of-input convention: an absent token is reported as an
*empty* span at the recorded `eof` offset rather than as no span at all, and
[`TokenCursor::here_span`] and [`slice_span`] must keep answering that the
same way for a diagnostic to point at the same place from either route.
