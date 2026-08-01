The frame cannot prove which contexts changed (the tree moved, the viewport
moved, a resource landed, ...), so every node is re-measured and the
retained [`MeasureCache`](crate::orchestration::MeasureCache) is dropped.
