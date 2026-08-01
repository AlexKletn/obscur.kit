Returns the node id this cursor points at.

Named `node_id` rather than `id` so it does not shadow the
[`MatchElement::id`] trait method (which returns the element's CSS `id`
attribute as `Option<&str>`).
