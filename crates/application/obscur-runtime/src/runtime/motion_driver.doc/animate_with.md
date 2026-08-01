[`animate`](Self::animate), plus the playback controls CSS's
`animation-delay`, `animation-iteration-count`, `animation-direction`,
`animation-fill-mode` and `transition-delay` set (§4.17).

The `playback` is lowered onto the [`Timeline`] this starts: the delay
arms [`Timeline::with_delay`] (a **negative** delay makes the timeline
seek that far into its play cycle at `start`, so it "appears to begin
part-way through" — css-transitions-1 §2.4), the repeat/direction/
alternation drive iteration parity, and the fill mode decides whether the
track's value is overlaid *during* the delay (css-animations-1 §3.9).
