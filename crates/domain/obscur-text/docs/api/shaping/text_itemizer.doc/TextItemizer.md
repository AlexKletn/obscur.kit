A port that splits text into uniform runs for shaping (spec §5.10).

An itemizer walks the source text in *logical* order and cuts it into
[`TextItem`]s that are each uniform in script and bidirectional direction.
The returned items tile the whole input: their byte ranges are contiguous,
non-overlapping, and together cover `0..text.len()` with no gaps. Splitting
happens before shaping so a [`TextShaper`](crate::shaping::TextShaper) only
ever sees a single-script, single-direction run.

The Unicode-driven adapter that consults real script and bidi tables lives
outside the domain; [`SingleRunItemizer`](super::SingleRunItemizer) is the deterministic, table-free
reference implementation used to exercise the port.
