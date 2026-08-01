The line index a caret byte `index` sits on within a wrapped `layout`
(spec §5.10): the first line whose byte range **ends past** `index`, else
the last line — so a caret at a soft-wrap/`\n` boundary sits at the start of
the *next* line and a caret at the text end sits on the final line. A
single-line layout always resolves to line `0`.
