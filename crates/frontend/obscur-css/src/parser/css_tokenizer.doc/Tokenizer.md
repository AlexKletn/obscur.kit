A hand-rolled CSS tokenizer (spec §4.13).

The tokenizer walks the source by [`char_indices`](str::char_indices), so it
never splits a UTF-8 character, and produces byte-accurate [`SourceSpan`]s.
It is *total*: it never panics. Comments (`/* … */`) are skipped; an
unterminated string or comment produces a recorded [`CssParseError`] while
still yielding a best-effort token stream so higher layers can recover.
