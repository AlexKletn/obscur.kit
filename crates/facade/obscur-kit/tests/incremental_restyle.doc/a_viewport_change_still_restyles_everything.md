A resize is not an interaction change: `@media` can re-match anything, so the
frame must fall back to resolving the whole tree rather than trust a dirty set
derived from pseudo-state alone.
