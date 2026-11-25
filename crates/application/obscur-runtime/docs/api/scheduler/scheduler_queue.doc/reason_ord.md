Returns the deterministic tie-break ordinal of a reason.

[`FrameReason`] deliberately does not implement [`Ord`], so the queue derives
a stable ordinal here to break ties between reasons that share a priority
band. This keeps ordering deterministic without imposing an ordering on the
reason type itself.
