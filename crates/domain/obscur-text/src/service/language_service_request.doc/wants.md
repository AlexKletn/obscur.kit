Returns whether `kind` is requested (spec §5.13).

An empty [`services`](Self::services) list requests every service, so this
returns `true` for any `kind` in that case; otherwise it returns whether
`kind` is explicitly listed.
