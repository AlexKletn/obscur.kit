A tolerance that permits nothing: byte-for-byte equality.

This is the honest default for a *same-backend determinism* check (the
same rasterizer, twice, on the same scene), which is not what §7.14's
tolerance is for — see [`ImageSnapshot::compare_within`](super::ImageSnapshot::compare_within).
