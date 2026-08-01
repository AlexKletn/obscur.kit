The same claim through the **primary** authoring surface: a `test-id` written
in `okui!` markup resolves to the node that carries it, and one command symbol
still means two things in two places.

Nothing about the routing changed to make this pass — [`macro_host`] is the
only difference from the case above, and it is a difference of front end. That
is the point: §7.5's identifier is now spellable on both, so positional
routing is a feature of the toolkit rather than of one file format.
