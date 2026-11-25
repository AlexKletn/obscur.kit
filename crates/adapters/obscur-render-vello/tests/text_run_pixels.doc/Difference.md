What two rasters disagree about.

`bytes` counts every differing channel, `visible_pixels` only those pixels whose
alpha is non-zero on at least one side — a pixel transparent in both rasters
reaches no screen whatever its colour channels hold — and `worst_alpha` is the
largest alpha gap found among them.
