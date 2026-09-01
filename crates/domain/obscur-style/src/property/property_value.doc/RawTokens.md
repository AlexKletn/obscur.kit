The declared value of a custom property, kept as the source text it was
written with.

A custom property's value is an arbitrary token sequence (CSS Variables 1
§2): it matches no grammar until `var()` substitutes it into a real
property, and only then is there a grammar to match it against. Lowering it
eagerly, as if it were an ordinary value, is what threw away everything past
its first component — `--font-body: Inter, sans-serif` kept `Inter` alone,
and kept it as a symbol minted by one sheet's interner, which no other sheet
can read.

So the text survives the cascade, and the substitution step parses it for
the property it lands on — through
[`CustomValueParser`](crate::computed::CustomValueParser), because the
grammar lives in the CSS frontend and this crate is domain. With no parser
installed there is no grammar to apply, and the substitution is invalid at
computed-value time like any other value the property cannot carry.
