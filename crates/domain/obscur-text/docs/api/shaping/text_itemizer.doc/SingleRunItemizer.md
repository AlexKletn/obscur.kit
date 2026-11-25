The trivial reference [`TextItemizer`](super::TextItemizer): the whole text as one run (spec §5.10).

`SingleRunItemizer` performs no script or bidi analysis at all. It returns
exactly one [`TextItem`] spanning `0..text.len()` (or `0..0` for an empty
string), with script [`ScriptTag::COMMON`], the given base direction, and
embedding level `0` for left-to-right or `1` for right-to-left. It exists so
the port can be tested and wired without any Unicode tables; the real,
table-driven itemizer arrives in a later step.
