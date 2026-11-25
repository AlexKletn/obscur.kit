Appends a group of commands that is already balanced.

Not `extend`, and the difference is the point: this leaves the depth and the
open-bracket stack alone, which is correct exactly when the group closes
everything it opens. A canvas painter's output is such a group by construction
— `CanvasRegion::close` guarantees it — and an unbalanced one would desync both
counters and corrupt every bracket after it.
