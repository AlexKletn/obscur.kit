The CSS `transform-origin`: the point a `transform` pivots about.

Each component is a [`Length`] — an absolute `px` or a `Percent` of the box's
border-box width (`x`) / height (`y`). The `left`/`center`/`right` and
`top`/`center`/`bottom` keywords lower to `0%` / `50%` / `100%`. The CSS
initial value is `50% 50%` (the box center), exposed as [`TransformOrigin::CENTER`];
an unauthored `transform-origin` is left unresolved and the coordinator
defaults to the center.

Because [`Length`] carries `f32` data, this derives [`PartialEq`] but not
[`Eq`]/[`Hash`].
