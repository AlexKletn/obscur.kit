Drops every cached parse, forcing the next [`install`](Self::install) to
re-parse from source text.

The escape hatch for an owner that edited a sheet without stamping a new
[`version`](CssSource::version) — §4.12's explicit invalidation.
