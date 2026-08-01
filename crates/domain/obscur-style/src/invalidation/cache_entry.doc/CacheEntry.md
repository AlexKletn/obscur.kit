A cached computed style tagged with the version it was produced at (§4.3).

Because [`ComputedStyle`] carries `f32` data, this derives [`PartialEq`] but
not [`Eq`].
