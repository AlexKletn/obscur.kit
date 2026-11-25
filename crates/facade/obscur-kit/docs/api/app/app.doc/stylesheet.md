Appends an authored stylesheet. Multiple calls accumulate in order; all are
installed (below the UA default sheet) through the motion-aware
[`CssPipeline`](crate::bootstrap::CssPipeline), so `@keyframes`/`transition`
work and CSS errors reach the diagnostics channel.

This is the *initial* set. To replace it while the application runs — a theme
switch, a re-read from disk — call
[`AppControl::set_stylesheets`](crate::control::AppControl::set_stylesheets) on
a [`control`](Self::control) handle: §4.12's explicit invalidation, which used
to be unreachable because the sheets were collected once and nothing ever
re-supplied them.
