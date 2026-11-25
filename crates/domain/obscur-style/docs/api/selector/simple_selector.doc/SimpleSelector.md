A single simple selector — the atomic unit of a compound selector.

This mirrors the parsed selector surface produced by `obscur-css`; a later
sub-step lowers a parsed compound into a sequence of these. Names are stored
as owned strings rather than interned symbols so that matching against tree
nodes (which intern in a *different* table than CSS) is a plain string
comparison — see the crate's interner-coherence note.
