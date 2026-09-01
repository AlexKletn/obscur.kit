Writes parsed components back out as CSS text.

Not a pretty-printer and not a serialiser for display: the one guarantee is
that [`lower_custom_value`] parses what this writes back into the components
it was given. Whitespace is normalised to a single space between components,
with none before a comma — the tokenizer treats runs of whitespace alike, so
the normalisation is not a loss.
