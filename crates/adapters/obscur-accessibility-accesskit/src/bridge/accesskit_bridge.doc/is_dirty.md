Returns whether an incremental [`apply_update`](Self::apply_update)
arrived since the last full push.

This adapter does not synthesise incremental AccessKit updates; a dirty
bridge signals the host to re-push a fresh [`SemanticSnapshot`] through
[`update_tree`](Self::update_tree), which clears the flag.
