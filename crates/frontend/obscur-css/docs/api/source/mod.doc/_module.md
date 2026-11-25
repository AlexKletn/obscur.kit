Source identity and byte spans for parsed CSS (spec §2.5 source maps).

A [`CssSource`] owns a stylesheet's [`SourceId`], logical name, and text;
parsers and diagnostics locate substrings within it using [`SourceSpan`]
byte ranges.
