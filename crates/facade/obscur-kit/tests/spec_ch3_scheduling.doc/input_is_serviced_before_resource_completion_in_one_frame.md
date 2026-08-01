§3.6 Priority order — "1. input, focus, pointer and IME; ... 4. resource
completion;".

With a key/pointer event and a resource completion both pending in one tick,
the log is `["input", "resource"]` — input first — in that single frame.
