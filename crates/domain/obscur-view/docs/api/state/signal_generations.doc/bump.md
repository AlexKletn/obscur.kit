Advances `source` to a new generation and returns it.

Called when the value behind the source changed. The toolkit does not diff
values to decide whether a change occurred — bumping *is* the statement
that it did, matching [`LocalState`](crate::state::LocalState)'s rule.
