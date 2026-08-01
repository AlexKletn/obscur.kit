A fully parsed CSS stylesheet with recovered rules and diagnostics.

Parsing is resilient: a malformed rule is recorded in `errors` and skipped,
and recognised-but-unsupported at-rules (for example `@import`) are recorded
in `unsupported` rather than aborting the parse.
