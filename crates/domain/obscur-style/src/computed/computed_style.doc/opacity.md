Convenience getter for the computed `opacity` as a number, if resolved.

Reads [`PropertyId::Opacity`] and returns the bare number it holds; it
returns [`None`] when opacity was not computed or (defensively) is not a
number.
