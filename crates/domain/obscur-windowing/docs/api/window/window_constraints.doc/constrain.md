Clamps `size` so each axis lies within the configured `[min, max]` range.

The maximum is applied first, then the minimum, so if a minimum exceeds
the corresponding maximum on an axis the minimum wins. This matches the
behavior of common platform window managers.
