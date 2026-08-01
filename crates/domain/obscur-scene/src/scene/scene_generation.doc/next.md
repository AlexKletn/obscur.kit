Returns the next generation, advancing past this one.

The returned generation compares strictly greater than `self` (barring
the extreme wraparound of the underlying [`VersionStamp::bump`]), marking
that the scene's content changed.
