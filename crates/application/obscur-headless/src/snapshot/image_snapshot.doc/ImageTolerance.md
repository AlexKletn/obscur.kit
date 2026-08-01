How much variation a raster golden comparison tolerates (spec §7.14).

# What the spec asks for

§7.14 is one sentence: *"Raster golden tests are allowed with explicit
tolerances for anti-aliasing and font/backend variation."* §5.2 adds
*"Golden tests use tolerances"* under its **allowed differences** list —
rasterization, anti-aliasing, font rendering details, backend-specific
precision.

Neither sentence prescribes a *metric*. What they pin down is that the
tolerance must be **explicit** and must absorb the named **sources** of
variation. This type therefore derives its shape from those sources rather
than from a number the spec never gives:

- **backend-specific precision** — a different blend or gamma rounding
  shifts *every* pixel by a *small* amount. Absorbed by
  [`max_channel_delta`](Self::max_channel_delta): a per-channel noise floor.
- **anti-aliasing and font rendering** — two rasterizers disagree only where
  coverage is *partial*: glyph and shape edges. That is a *small fraction* of
  pixels, but on those pixels the delta is *unbounded* — at a hard black-on-
  white edge, 49% vs 51% coverage is a full swing. Absorbed by
  [`max_differing_fraction`](Self::max_differing_fraction): a cap on *how
  many* pixels may exceed the noise floor, with no cap on by how much.

# Why two knobs and not one

The two sources are opposite in shape — many pixels × small delta versus few
pixels × large delta — so no single scalar grants both permissions. Each of
the one-knob designs was measured against real renders in
`obscur-render-cpu`'s `golden_tolerance.rs`, and each fails, in *opposite*
directions:

- **A lone noise floor is too loose.** Passing an AA edge needs a floor of
  240, and a floor of 240 also passes an 8/255 recolouring of the entire
  fill — a real regression waved straight through.
- **A lone differing-pixel cap is too strict.** With no floor, the 4/255
  rounding difference §5.2 explicitly permits moves 52% of the image into
  "differing", so no cap that still rejects regressions can accept it.
  "Differs at all" cannot tell a rounding wobble from a wrong colour.

Together each knob grants exactly one of the two permissions, and a
difference that is neither shape is rejected.

# A deliberate omission

There is **no cap on the channel delta of a differing pixel**, and that is
not an oversight: legitimate AA variation on a partial-coverage pixel really
can span the full channel range, so such a cap would reject the very
difference §5.2 permits.

# What this therefore does not catch — read before trusting it

A regression with a **small footprint** passes. The two knobs bound *how far*
a pixel may move and *how many* may move, and nothing bounds where: a 10×10
element dropped from a 64×64 surface moves 100 pixels — 2.44% — at a full
255 delta, and is accepted by any fraction above that, though §5.2 puts scene
geometry under *must match*. This was measured, not reasoned about.

The earlier version of this doc claimed the opposite ("a real regression moves
more pixels than the tolerated fraction"). It does not follow, and it is the
worse error: a reader trusting it concludes they are guarded where they are
not. The fraction cap protects against a *diffuse* regression, and only that.

Closing it needs a discriminator neither knob has, because the honest
difference between AA noise and a dropped element is **shape**, not magnitude
or count: AA differences hug an outline and are a pixel or two wide, while a
dropped element is a contiguous filled region. A per-tile fraction, or a
reject-if-any-NxN-block-differs rule, would express that — both need a size
constant this workspace has no evidence to pick yet, so neither is guessed at
here.
