The vertical scroll offset in **physical** px (mouse-wheel driven), clamped
to `0 ..= max(0, content_height*effective_scale - physical_height)`. Pushed
to the runtime as a logical content offset (`scroll_y / effective_scale`).
