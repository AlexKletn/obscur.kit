Classifies the phases a frame produced for `reason` must run.

When `first_frame` is set (the retained tree has never been built) the
plan is [`full`](Self::full) regardless of reason: the first frame — and
any frame over an empty tree — must run every phase so the tree, styles,
boxes and scene all exist before later frames can reuse them.
