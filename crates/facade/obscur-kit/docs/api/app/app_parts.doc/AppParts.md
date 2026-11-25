What an [`App`](super::App) becomes once its root has been resolved.

Not a config object an application ever sees — it exists because the windowed
and desktop-surface runners need the same ten things, and a positional argument
list that long is how a caller comes to pass the virtual-click handlers where
the long-press handlers go.
