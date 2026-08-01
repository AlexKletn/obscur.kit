Builds the full capabilities view tree. Pure view/primitive construction — no
backend types, no state — so every driver calls it identically. The `cpu`
flag only swaps two captions (the gradient/blur cards) so the CPU PNG stays
honest about the GPU-only effects; the geometry is identical.
