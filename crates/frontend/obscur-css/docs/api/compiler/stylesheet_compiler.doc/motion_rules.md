Selector-associated motion: per style rule, the `transition` templates
and resolved `@keyframes` animations that rule declares, paired with the
selector list the runtime matches against the retained tree (§4.17).

This is the wiring input the runtime consumes to drive CSS transitions and
animations: unlike the flat [`transitions`](Self::transitions) /
[`keyframes`](Self::keyframes) outputs, each entry keeps the association
between a selector and the motion it declares, so a transition fires only
on the nodes its rule matched and a `@keyframes` plays only on elements
that reference it via `animation`/`animation-name`.
