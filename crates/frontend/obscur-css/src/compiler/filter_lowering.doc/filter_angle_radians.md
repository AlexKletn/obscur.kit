Parses one `hue-rotate` `<angle>` argument into **radians**.

Accepts `deg`, `grad`, `rad`, and `turn` dimensions, a bare `0` (`Number(0)`),
or an empty argument list (the CSS initial `0deg`). Returns [`None`] for more
than one argument, a non-zero bare number, a percentage, or an unknown unit.
