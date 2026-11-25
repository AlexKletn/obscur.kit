Returns the installed hyphenation port, or [`None`] when hyphenation is
off (the default).

The layout phase measures text through this so it wraps (and hyphenates)
identically to the compose phase, keeping measured box height and painted
line count in agreement.
