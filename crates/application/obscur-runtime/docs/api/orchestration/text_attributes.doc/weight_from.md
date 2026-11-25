Narrows a CSS numeric weight into a [`FontWeight`] on the `1..=1000` axis.

The value is rounded and clamped to the representable range; [`FontWeight::new`]
clamps again, so an out-of-range author value never panics.
