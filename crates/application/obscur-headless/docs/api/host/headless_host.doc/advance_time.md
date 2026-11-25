Advances the deterministic clock by `nanos` nanoseconds **and drives the
runtime's motion by the same amount**.

Time in a headless run only moves when the test asks it to. The delta
handed to motion is derived from the clock's own readings rather than from
`nanos`, so the clock — not this argument — is what a frame's animation
progress follows.
