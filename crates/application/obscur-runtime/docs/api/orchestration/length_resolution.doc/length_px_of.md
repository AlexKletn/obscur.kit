Resolves one length-valued property into a pixel value, defaulting to `0.0`.

Layout insets and `gap` are pixel-only, so a bare `Percent` or `Auto` length
(or an absent/non-length property) contributes `0.0`; relative/viewport units
and `calc()` are folded to pixels against `ctx`.
