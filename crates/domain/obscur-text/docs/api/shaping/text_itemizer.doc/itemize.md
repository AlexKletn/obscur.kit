Splits `text` into uniform script + direction runs in logical order,
seeded by the paragraph `base_direction`.

The returned items cover `0..text.len()` contiguously with no gaps or
overlaps. An empty `text` yields a single empty item `0..0`.
