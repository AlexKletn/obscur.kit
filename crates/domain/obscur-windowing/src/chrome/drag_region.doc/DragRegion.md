A semantic window-drag area, such as a custom title bar (spec §6.4).

A compositor treats clicks within [`Self::bounds`] as a request to move the
window, and double-clicks as the configured [`DoubleClickAction`].
Coordinates are in window-local logical pixels.
