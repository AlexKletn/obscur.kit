The field's horizontal scroll offset (logical px, ≥ 0): how far the text
run is scrolled left within the field's content box so the caret stays in
view (the single-line text-control scroll model, CSS Overflow 3 §3.1
scroll-into-view). Recomputed after every caret move/edit by
[`UiRuntime::update_caret_scroll`](super::UiRuntime::update_caret_scroll)
and read back by the compose phase (via
[`EditingView::scroll_left`](crate::orchestration::EditingView::scroll_left))
and the click inverse. `0.0` for a value that fits its box and for any
wrapping (multi-line) field, which never scrolls horizontally.
