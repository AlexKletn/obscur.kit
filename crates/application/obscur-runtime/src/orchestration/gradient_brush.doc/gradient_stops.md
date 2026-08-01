Maps a style gradient's color stops to scene [`SceneGradientStop`]s: each stop's
resolved `0.0..=1.0` position becomes the scene offset and its
[`StyleColor`](obscur_style::property::StyleColor)
the scene paint color. An unpositioned stop (never produced by the CSS lowering,
which resolves every position) falls back to its index-proportional offset.
