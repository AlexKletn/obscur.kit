Emits the §5.2 explicit fallback diagnostic for the effects a backend will not
reproduce, once per effect.

A backend owns one of these and calls [`report`](Self::report) from its render
path with the [`unsupported_effects`](crate::renderer::unsupported_effects) of the scene it is about to draw. Whether
a backend reproduces an effect is a property of the **backend**, not of the
frame, so each effect is reported the first time a scene demands it and never
again: a host learns "this backend will not draw your gradients" once, instead
of sixty times a second for as long as the gradient is on screen.

The diagnostic is [`Warning`](DiagnosticSeverity::Warning) with
[`RecoveryAction::Fallback`]: the frame is still drawn, with the backend's
documented approximation in place of the effect.
