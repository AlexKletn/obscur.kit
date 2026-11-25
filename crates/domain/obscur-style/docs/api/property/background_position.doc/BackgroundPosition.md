The computed value of `background-position`: where the (already sized) image
tile's origin sits within the painted box, one [`Length`] per axis.

Each component is stored as a [`Length`] the scene coordinator resolves at
paint time against the box, following the standard CSS position formula:

* a `px` offset places the tile that many pixels from the box's top/left edge;
* a `%` offset aligns the image's `pct` point to the box's `pct` point — i.e.
  the tile origin is `(box_extent − tile_extent) × pct` from the edge (so
  `50%` centers and `100%` right/bottom-aligns);
* the position keywords lower to percentages: `left`/`top` → `0%`,
  `center` → `50%`, `right`/`bottom` → `100%`.

The CSS initial is `0% 0%` (top-left), i.e. both axes [`Length::Percent(0.0)`].

Because [`Length`] carries `f32` data this derives [`PartialEq`] but not
[`Eq`]/[`Hash`].
