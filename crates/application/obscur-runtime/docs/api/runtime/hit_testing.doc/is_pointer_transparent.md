Whether the node's computed visibility is hidden or its computed
`pointer-events` is `none`.

Both properties inherit. A descendant can override the inherited value, so
this check reads the candidate's computed style while ancestor geometry still
applies. Missing values default to visible and pointer-targetable.
