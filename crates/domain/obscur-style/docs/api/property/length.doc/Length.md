A CSS length value.

Absolute (`Px`) and percentage (`Percent`) terms are self-contained; the
font-relative (`Em` / `Rem` / `Ch`) and viewport-relative (`Vw` / `Vh` /
`Vmin` / `Vmax`) terms carry their coefficient and only become a concrete
number once resolved against a [`LengthContext`] via [`Length::resolve_px`].

Because the variants carry `f32` data, this type derives [`PartialEq`] but
not [`Eq`]/[`Hash`].
