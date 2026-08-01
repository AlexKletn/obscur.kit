The arrange walk shared by every entry point: arranges the root under a
tight `viewport` constraint and returns every reachable node's absolute box,
measuring through `cache`.

Sharing this walk is what makes the full and incremental paths agree by
construction rather than by coincidence: they differ **only** in what
`cache` already holds when it starts.
