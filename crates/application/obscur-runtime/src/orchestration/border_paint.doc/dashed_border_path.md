Builds the path a uniform dashed border strokes along: the border box inset by
`width / 2` on every side, so a stroke of `width` centered on it fills the CSS
border band exactly (CSS borders sit inside the border box). Rounded when the
element has a corner radius (each radius reduced by `width / 2`, the centerline
radius), else a plain rectangle.
