Resolves every stop's used position to an explicit `0.0..=1.0` value (spec
§5.2 color-stop placement).

The first stop defaults to `0.0` and the last to `1.0` when unauthored;
authored positions are clamped to be non-decreasing (a position is at least the
previous stop's); and each run of unpositioned stops is spread evenly between
its surrounding positioned neighbours. After this, every stop carries a `Some`
position.
