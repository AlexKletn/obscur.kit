Computes the ordered propagation path from a root → target ancestor chain.

`ancestors_root_to_target` is the path from the ROOT down to and
INCLUDING the target as its last element. The result is: a
[`Capture`](EventPhase::Capture) step for each ancestor above the target
(root → …), then a single [`Target`](EventPhase::Target) step for the
target, then a [`Bubble`](EventPhase::Bubble) step for each of those
ancestors in REVERSE (… → root).

An empty input yields an empty path; a single element (the target alone)
yields just `[Target]`.
