The largest number of image copies drawn per axis for a tiled
(`background-repeat`) background — a bounded-loop guard against a pathological
tiny-tile / huge-region combination. The visible output is clipped to `region`
regardless, so this only caps wasted work (documented limitation).
