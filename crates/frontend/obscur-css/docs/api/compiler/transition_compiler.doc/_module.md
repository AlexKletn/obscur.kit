Lowering CSS `transition` into `obscur-motion` track templates (§4.17).

A CSS `transition` names a property, a duration, an optional delay, and a
timing function — but *not* the endpoints, which are only known at runtime
(the value before and after a state change). [`TransitionCompiler`] therefore
lowers a `transition` declaration into a [`CompiledTransition`] *template*:
the property, timings (in seconds), and an [`Easing`]. At runtime,
[`CompiledTransition::build_track`] binds concrete `from`/`to`
[`MotionValue`]s into a [`TransitionTrack`].

`cubic-bezier(...)` / `steps(...)` timing functions are not modelled by the
domain's [`Easing`] enum; they fall back to [`Easing::Linear`] and are
reported as an [`UnsupportedCssFeature`] (§4.13).
