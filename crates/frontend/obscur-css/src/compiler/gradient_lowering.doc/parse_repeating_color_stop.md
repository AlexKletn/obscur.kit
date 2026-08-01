Parses one `repeating-linear-gradient` color stop into `out`, supporting the
`<color>`, `<color> <pos>`, and hard-band `<color> <pos> <pos>` forms. The
hard-band form pushes the color twice (once at each position). Returns [`None`]
for an unparseable color or position.
