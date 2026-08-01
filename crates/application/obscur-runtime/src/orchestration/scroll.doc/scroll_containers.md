Enumerates every scroll container in the tree that **currently overflows**,
depth-sorted ascending (outermost first) so a consumer can compose their
scroll positions outer-to-inner (CSS Overflow 3 §scroll-container).

A scroll container whose content fits (content height ≤ scrollport height)
owns no scroll position and is omitted — with `overflow: auto` it also shows
no scrollbar, matching the spec's "as needed" rule.
