Records one frame's CPU and present durations. Returns a formatted
`"N fps · cpu …ms · gpu …ms · frame …ms (worst …ms)"` line **once per
interval** (the wall-clock present FPS plus the rolling timing breakdown),
or `None` on the frames in between.
