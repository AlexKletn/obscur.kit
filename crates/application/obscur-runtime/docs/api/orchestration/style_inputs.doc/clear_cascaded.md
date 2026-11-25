Drops `node`'s cascaded declarations, so it resolves to its
default/inherited computed style again.

This is the counterpart [`set_cascaded`](Self::set_cascaded) needs once
inputs are rebuilt for *some* nodes rather than from scratch: a node that
matched a rule last frame and matches none this frame must lose its entry,
not keep the stale one.
