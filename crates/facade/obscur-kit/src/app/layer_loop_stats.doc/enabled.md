A counter if `OBSCUR_FRAME_STATS` is set, and nothing otherwise.

`Option` rather than a silent no-op so the timing calls themselves are skipped
when nobody asked, and the loop pays nothing for an instrument it is not using.
