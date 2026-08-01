Every way a raster golden comparison fails to hold (spec §7.14).

This deliberately folds *"compared, and differs too much"* together with
*"could not compare at all"*. A golden check that cannot run is not a golden
check that passed: if `PixelsUnavailable` were reported as success, a
snapshot rebuilt from a bare hash would wave every regression through.
