The smallest permitted iteration duration, in logical seconds.

A non-positive duration would make the `progress = elapsed / duration`
division ill-defined, so [`new`](Self::new) clamps up to this tiny
positive epsilon instead.
