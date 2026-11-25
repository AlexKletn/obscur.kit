The content of the first text node in the retained tree.

What a rebuilt view actually put on screen, read back in one line. The tests
that use it drive a *reactive* runtime — the app mutates its own state, the held
view builder runs again, and this is the assertion that the round trip closed:
the value the handler wrote is the value the tree now carries.

"First" is not a weakness of the selector but a property of the fixtures it
reads: each of them renders exactly one text node, so the first is the only one.

# Panics

If the retained tree holds no text node, which means no frame has run.
