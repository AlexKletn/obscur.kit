Adds `overflow: auto`, which is what establishes a scroll container.

CSS Overflow 3: without it, content taller than its box overflows visibly and the
runtime records no scroll state at all — so a fixture that forgets this scrolls
nothing and reports no error.
