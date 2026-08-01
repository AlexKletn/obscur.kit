What the runtime knows about the frame that an app needs in order to size or
window its own content.

A view builder is otherwise blind: it cannot tell how tall the window is, so it
cannot decide how many rows are worth building. That is the whole prerequisite
for app-side virtualization — a list of 10,000 entries costs style, layout and
compose per node, so the only way to stay cheap is to not build the nodes.
