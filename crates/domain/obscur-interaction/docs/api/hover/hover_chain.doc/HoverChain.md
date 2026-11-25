The ordered chain of nodes currently under the pointer, topmost-first
(hover).

This parallels the [`HitChain`](crate::spatial::HitChain): index `0` is the
topmost node the pointer is over (the hover target); the last element is the
root. Comparing successive hover chains yields the enter/leave diff via
[`HoverTransition`](super::HoverTransition).
