Writes one parsed component, appending to `out`.

A quoted string is re-quoted and a function's arguments go back through
[`custom_value_text`] recursively; everything else is written as the tokenizer
would read it back.
