Reports the new ends through the node's `on:input`, and says whether the gesture was a slider's.

A control that did not move reports nothing: a drag along a stepped track spends
most of its samples inside one step, and an application re-rendering on every
one of them would be paying for motion nobody can see.

It answers `true` even when nothing is bound, because the question it answers is
"was this a slider" and not "did anyone listen".
