The grammar the computed-value step cannot own: [`CustomValueParser`].

A custom property is declared as an arbitrary token sequence and stored as
[`PropertyValue::RawTokens`](crate::property::PropertyValue::RawTokens).
Turning that text into a value takes the CSS grammar — the family list of
`font-family`, the shadow list of `box-shadow`, the `<color>` of every colour
longhand — and that grammar lives in the CSS frontend, which is layered
*above* this crate and may not be depended on from it.

So the resolver asks for the parse through this port, exactly as it asks for a
`ch` measurement through [`ChMetrics`](super::ChMetrics). With no parser
installed there is no grammar to apply, and a raw token sequence is invalid at
computed-value time — which is `unset`, the same answer a value of the wrong
shape gets.
