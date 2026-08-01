Classifies an effective [`AlignItems`] into a grid [`CellAlign`]. The flexbox
spellings (`FlexStart` / `FlexEnd`) fold onto the grid `Start` / `End`
physical alignments so an authored `align-items: start` (which the runtime
maps to [`AlignItems::FlexStart`]) behaves as a grid start.
