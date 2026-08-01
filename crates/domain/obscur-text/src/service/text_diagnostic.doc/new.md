Creates a diagnostic flagging `range`, stamped with `version` (spec §5.13).

The diagnostic starts with no [`suggestions`](Self::suggestions); attach
them with [`with_suggestions`](Self::with_suggestions) or
[`with_suggestion`](Self::with_suggestion).
