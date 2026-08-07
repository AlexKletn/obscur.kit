Drops `node`'s cascaded custom properties, so it declares none of its own
and sees only what it inherits.

The counterpart [`set_custom`](Self::set_custom) needs on the incremental
path, for the same reason [`clear_cascaded`](Self::clear_cascaded) exists: a
node that declared `--brand` last frame because it matched `.dark`, and
matches nothing this frame, must lose it — and every descendant that
inherited it must lose it too.
