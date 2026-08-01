Resolves each repeating stop's used magnitude in its period unit (spec §5.2
color-stop placement, adapted for a repeating gradient's unnormalized axis).

The first stop defaults to `0`; authored positions are clamped non-decreasing;
runs of unpositioned stops are spread evenly between their positioned
neighbours. Returns [`None`] when the last stop is unpositioned (a repeating
gradient needs it to define the period).
