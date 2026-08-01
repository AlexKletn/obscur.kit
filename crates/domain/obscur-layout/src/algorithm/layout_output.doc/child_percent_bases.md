Parallel to `child_boxes` when stated, and **empty** when the algorithm
stated nothing (the overwhelmingly common case, and the reason this is not
a `Vec<Option<…>>` sized to match): an out-of-range index reads as
[`ChildPercentBasis::UNSTATED`].
