Re-clamps `scroll_y` into `0 ..= max_scroll_y()` and pushes the resulting
**logical** content offset (`scroll_y / effective_scale`) to the runtime, so
a physical wheel step scrolls a consistent on-screen amount at any scale.
