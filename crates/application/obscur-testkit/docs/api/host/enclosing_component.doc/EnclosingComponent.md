A component whose only job is to render a
[`GuardedComponent`](super::GuardedComponent) that cannot decode its props.

It exists so a failure can originate strictly *below* a boundary. With the inner
component's boundary disabled, the failure has nowhere to be contained but this
one — and what the test then reads is which subtree got replaced and which node
each diagnostic was anchored to. Its own fallback says "outer fallback" so the
two are never confused on screen.

It takes no props, so nothing about *it* can fail: the only failure in the tree
is the one the fixture is arranging.
