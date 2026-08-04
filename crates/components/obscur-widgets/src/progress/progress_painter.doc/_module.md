The one painter every progress bar shares.

It reads its fraction and its shape off the node it is drawing — which is why
`CanvasPainter` is handed the node's properties. Without them one painter could
not serve two bars at different values.
