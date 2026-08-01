Oklab triple from **linear-light** sRGB (Björn Ottosson's Oklab, adopted by
CSS Color 4 §9). Inputs are linear sRGB (no transfer function applied);
returns `[L, a, b]` with `L` nominally `0.0..=1.0` and `a`/`b` signed. This is
the exact inverse of [`oklab_to_linear_srgb`] and shares its canonical
matrices; densifying a gradient's stops in Oklab needs this forward direction.
