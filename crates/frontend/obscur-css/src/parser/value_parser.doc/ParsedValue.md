A single parsed CSS component value (spec §4.13).

This is a *syntactic* model: no unit validation, colour resolution, or
arithmetic happens here. Commas and bare operator characters are preserved as
[`Comma`](ParsedValue::Comma) and [`Delimiter`](ParsedValue::Delimiter)
components so a value round-trips structurally into the compiler.
