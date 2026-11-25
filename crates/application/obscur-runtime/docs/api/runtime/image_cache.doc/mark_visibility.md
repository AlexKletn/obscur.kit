Sets each resident image's eviction priority from `referenced` — this
frame's on-screen verdict per URL — defaulting a URL no node references at
all to [`VisibilityPriority::Hidden`] (spec §5.19 visibility priority).

An image the tree stopped referencing (a closed page, a swapped route) is
the one eviction should take first, and `Hidden` is what says so under
[`EvictionPolicy::VisibilityAware`].
