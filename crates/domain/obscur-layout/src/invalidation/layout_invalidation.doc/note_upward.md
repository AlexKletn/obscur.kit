**Upward gate.** When a child's `child_result` continues upward
([`PropagationResult::continues_upward`]), invalidates `parent_ctx`'s
node with
[`ChildSizeChanged`](LayoutInvalidationReason::ChildSizeChanged) and
returns `true` so the runtime knows to keep walking up. If the child's
geometry was stable or contained, nothing is recorded and `false` is
returned.

This encodes §4.3's "propagate up only when output geometry changes" and
the containment stop, delegating the decision to the child's
[`PropagationResult`].
