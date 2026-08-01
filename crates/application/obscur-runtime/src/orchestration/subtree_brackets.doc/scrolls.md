A scroll container's descendant translation `(node, scroll_top)`, nested
INSIDE its overflow clip and OUTSIDE its own paint, so the content scrolls
within the clipped scrollport while the container's own background stays
fixed (CSS Overflow 3 §scrollport). A zero offset is never recorded (it is
the identity), so an unscrolled container is byte-identical save for its
scrollbar.
