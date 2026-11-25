A compiled selector: its cascade [`SelectorId`] and its specificity.

The parsed selector AST carries no per-token spans, so the excluded
pseudo-classes it triggers are reported against an empty span.
