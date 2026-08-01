Whether a timeline applies its value outside the period it is actually
running (css-animations-1 §3.9).

The only phase this engine distinguishes is the **delay** period before the
first iteration: with [`Backwards`](Self::Backwards) or [`Both`](Self::Both)
the value of the keyframe that will start the first iteration is applied
during the delay; with [`None`](Self::None) or [`Forwards`](Self::Forwards)
nothing is applied until the first iteration begins.

**Limitation.** The after-phase is *not* distinguished: a finished animation
keeps applying the value it ended on regardless of this mode, so
[`None`](Self::None) and [`Forwards`](Self::Forwards) behave identically once
the last iteration completes — `forwards` is satisfied by construction and
`none` over-applies. Only the backwards half of the property is modelled, and
only [`fills_backwards`](Self::fills_backwards) has a consumer.
