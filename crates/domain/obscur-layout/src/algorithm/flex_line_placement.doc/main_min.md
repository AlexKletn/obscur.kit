Pre-resolved main-axis min floor / max ceiling per item, indexed by the
global item index (`main_max` is `f32::INFINITY` when the item has no
`max`). `main_min` folds in the §4.5 content-based automatic minimum on the
`auto` path, so the freeze loop shrinks items no further than their content.
