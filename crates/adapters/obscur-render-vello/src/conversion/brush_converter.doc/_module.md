Converts obscur-scene paint sources ([`Brush`], [`Stroke`]) into peniko/kurbo.

Pure, backend-local conversion (spec §2.7). Solid and gradient brushes map to
[`peniko::Brush`]; stroke style maps to a [`kurbo::Stroke`]. Stop colors and
offsets, and the stroke's width/join/cap, are preserved exactly (spec §5.2 —
the adapter keeps paint data authoritative and delegates only rasterization).
