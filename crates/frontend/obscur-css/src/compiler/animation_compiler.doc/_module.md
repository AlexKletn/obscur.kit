Lowering CSS `animation-*` declarations into `@keyframes` references (§4.17).

Unlike `transition`, an `animation` names one or more `@keyframes` rules to
play on the element plus a duration. [`AnimationCompiler`] extracts, from a
style rule's declarations, the list of [`AnimationRef`]s it declares: an
interned animation name (resolved against the stylesheet's compiled
`@keyframes` later, at [`compile`](crate::compiler::StylesheetCompiler::compile)
time) paired with its duration in seconds.

Both the `animation` shorthand (`animation: pulse 1s ease-in`) and the
longhands (`animation-name: pulse; animation-duration: 1s`) are recognised,
as are `animation-delay`, `animation-iteration-count`, `animation-direction`
and `animation-fill-mode` — each lowered onto the [`AnimationRef`] the
runtime turns into a [`Timeline`](obscur_motion::timeline::Timeline).
`animation-play-state` and `animation-timing-function` are still ignored.
