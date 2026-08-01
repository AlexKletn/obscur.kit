The item count a `<VirtualFor>` window is sized against when the collection's
real length is unknown at build time (spec §4.9). Deliberately far larger
than any viewport can show, so it never bounds the window: the window is
bounded by the viewport, which is the whole point of virtualization.
