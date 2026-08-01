Parses one `<angular-color-stop>` for a `conic-gradient`: a lone color
(auto-positioned) or a color followed by an `<angle-percentage>` position
(css-images-4 §3.4). Unlike a linear/radial stop, a conic stop's position may be
an `<angle>`, normalized against a full turn (`0deg` → `0.0`, `360deg` → `1.0`);
a `<percentage>` is also accepted. Returns [`None`] for an unparseable color or
an out-of-grammar position.
