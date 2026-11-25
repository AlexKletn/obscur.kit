Returns the explicitly allowed child tags — the [`Only`](Self::Only) list,
or an empty slice for the two policies that name no tags.

An empty slice is therefore *not* a permissive answer: ask
[`admits`](Self::admits) or match the policy, never
`tags().is_empty()`.
