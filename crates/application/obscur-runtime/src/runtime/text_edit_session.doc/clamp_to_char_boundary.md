Clamps a byte `offset` into `text`: capped at `text.len()` and floored to the
nearest UTF-8 char boundary at or below it, so the result is always a valid
caret position in `text` (an offset from a previous, differently-sized value
may land past the end or mid-codepoint).
