The computed-value-time judgement on a substituted `var()`: the value itself
when it fits `property`, and [`PropertyValue::Unset`] when it does not.

A custom property declared as plain text arrives here as
[`PropertyValue::RawTokens`](crate::property::PropertyValue::RawTokens) and is
read against `property`'s grammar first, through `custom_values` — the point
in the pipeline where a token sequence finally has a grammar to be read
against. Text that parses to nothing, and text with no parser installed to
read it, are both invalid at computed-value time.

`Unset` is routed through the ordinary specified-value path, so an inherited
property takes its parent's computed value and a non-inherited one takes its
initial — exactly as if the declaration had said `unset` in so many words.
