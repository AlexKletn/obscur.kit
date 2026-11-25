The blur radius in logical pixels, or [`None`] when this is not a pending blur
or its radius is still unknown.

A caller that gets [`None`] for a pending blur must leave the list unresolved —
substituting `0` is the defect this type exists to remove.
