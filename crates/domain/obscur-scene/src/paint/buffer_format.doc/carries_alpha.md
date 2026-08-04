Whether the fourth byte is alpha rather than padding.

False for every `x` and every three-byte layout: those are opaque, and their
declared [`AlphaMode`](crate::color::AlphaMode) means nothing.
