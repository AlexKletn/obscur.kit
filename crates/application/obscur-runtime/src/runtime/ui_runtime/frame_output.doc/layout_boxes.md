Returns the absolute layout boxes retained from the last produced frame,
each translated into **viewport-relative** space by the scroll chain above
it (spec §4.4 geometry observers): a node inside an overflowing scroll
container reports the position it is painted at, with every scroll-container
ancestor's `scroll_top` folded out. A node under no scrolled ancestor —
including a scroll container reporting its own box — is unchanged, so an
unscrolled tree returns the content-space boxes untouched.

Empty until the first frame runs. The runtime's own paint/compose and
hit-test do **not** read this projection; they fold scroll themselves from
the retained content-space boxes.
