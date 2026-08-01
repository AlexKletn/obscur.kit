End-to-end coverage for audit S4 (§4.17): a CSS-authored `transition` and
`@keyframes` animation, compiled through the real CSS frontend and driven by
the runtime's `MotionScheduler`, actually run.

This exercises the whole wiring through the public facade only: author CSS ->
hand the sources to the kit's [`CssPipeline`] -> render frames and observe the
animated value flow through style -> motion -> layout. The CSS->runtime
plumbing is the kit's, not the test's: a host installs sources and renders.
