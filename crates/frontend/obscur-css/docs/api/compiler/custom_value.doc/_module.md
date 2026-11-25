A custom property's value, out of the compiler as text and back in as a
value.

CSS Variables 1 §2 says a custom property's declared value is an arbitrary
token sequence: it matches no grammar, because until `var()` puts it somewhere
there is no grammar it could match. Lowering it eagerly, the way an ordinary
declaration is lowered, is a category error — and it lost real values, keeping
only the first component of `--font-body: Inter, sans-serif` and only the `0`
of `--shadow: 0 2px 4px #000`.

So this module holds the two halves of the deferral:

* [`custom_value_text`] writes the parsed components back out as the text they
  were written as, which is what the declaration carries through the cascade
  (as [`PropertyValue::RawTokens`]).
* [`lower_custom_value`] reads that text again at substitution time, for the
  property it landed on — the first moment a grammar exists to read it
  against. The style resolver reaches it through the
  [`CustomValueParser`](obscur_style::computed::CustomValueParser) port,
  because the resolver is domain and this grammar is frontend.

The round trip goes through text rather than through a shared component type
because the value has to cross a layer boundary: the domain holds the
declaration, and it cannot be taught the frontend's token model to do it.
