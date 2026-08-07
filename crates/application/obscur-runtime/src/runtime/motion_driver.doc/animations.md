What each timeline animates: its node, its property, its track and how the
sampled scalar lowers to a value.

A binding outlives its timeline — that is what holds the endpoint value on
screen after the scheduler prunes a finished animation — so the only thing
that ends one is being superseded for the same target, or its node leaving the
tree ([`retain_live`](Self::retain_live)). Every frame's
[`apply_overlay`](Self::apply_overlay) walks this whole vector, so its length
is frame time, not just memory.
