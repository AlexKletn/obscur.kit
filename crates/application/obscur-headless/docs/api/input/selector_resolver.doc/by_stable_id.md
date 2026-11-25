The unique live node whose `attribute` property equals `id`.

Shared by `test-id` and `automation-id`: they differ only in which symbol
they read, so one lookup keyed by the symbol keeps them from drifting — and,
because the symbols are distinct, a target for one never matches the other.

Both are read from the node's retained **properties** rather than from its
style attributes, which is what keeps §7.5's "does not affect CSS
specificity / layout / visual output" true by construction.
