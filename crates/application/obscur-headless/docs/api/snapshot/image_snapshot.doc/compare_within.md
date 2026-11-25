Compares this render against a golden `other` under an **explicit
tolerance** (spec §7.14).

`self` is the golden and `other` the render under test; the comparison is
symmetric, so the roles only shape the error's `expected`/`actual`.

Returns the measured [`ImageDiff`] when the render is within tolerance —
including how much headroom was left — and a [`GoldenMismatch`] for every
way the check does not hold, *including* the ways it could not run at all.

# Errors

- [`GoldenMismatch::DimensionMismatch`] — the rasters are different sizes.
- [`GoldenMismatch::ByteLengthMismatch`] — same size, different byte counts.
- [`GoldenMismatch::PixelsUnavailable`] — a side carries only a hash.
- [`GoldenMismatch::TooManyDifferingPixels`] — too much of the image moved.
