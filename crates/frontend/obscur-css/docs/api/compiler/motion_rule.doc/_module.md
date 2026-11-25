Selector-associated motion: [`CompiledMotionRule`] and [`CompiledAnimation`].

A CSS `transition` or `animation` is a property *of the elements a rule
matches* — a `transition` says "when **my** property changes, ease it", and
an `animation` plays a named `@keyframes` on the elements the rule selects.
The flat [`transitions`](super::CompiledStylesheet::transitions) and
[`keyframes`](super::CompiledStylesheet::keyframes) outputs lose that
association, so the runtime cannot know *which* nodes a transition/animation
applies to.

[`CompiledMotionRule`] restores it: it pairs a rule's authored
[`SelectorList`] with the transition templates and resolved keyframe
animations that rule declares, so the runtime can match the selector against
the retained tree and drive the motion on exactly the matched nodes (§4.17).
