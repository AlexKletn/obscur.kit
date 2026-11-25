How many entries the four `NodeId`-keyed bookkeeping maps hold, as
`(settled_progress, transition_targets, transition_runs, css_animations_started)`.

Test-only: no shipped surface reads it. It exists so a test can state the
property [`retain_live`](Self::retain_live) is *for* — that an unmounted node
leaves nothing behind — as an assertion about sizes, which is what a leak
actually is. Nothing here is observable from the outside, and that is exactly
why it went unnoticed.
