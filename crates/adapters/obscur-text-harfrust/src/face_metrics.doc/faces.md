Per-face design metrics in registration order, indexed by
[`FontId::index`]; `None` for a face whose real metrics could not be read.
The whole table is optional so [`empty`](FaceMetrics::empty) — the state of
a shaper that was handed no registry — is `const`-constructible.
