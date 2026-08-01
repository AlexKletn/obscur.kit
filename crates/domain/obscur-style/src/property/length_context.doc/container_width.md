The base a `cqw` resolves against, in logical pixels.

css-contain-3 §6: *"The query container for each axis is the nearest
ancestor container that accepts container size queries on that axis. If
no eligible query container is available, then use the small viewport
size for that axis."*

# What this engine can and cannot answer

**No element here ever accepts container size queries.** There is no
`container-type` property (see `docs/conformance-audit.md`, item 4-B:
`@container` is parsed, fails closed, and is inert, because a container's
size is a layout *output* while style matching runs strictly before
layout). So the "no eligible query container" branch is not an edge case
here — it is the **only** branch, and every container unit resolves
against the viewport. The *resolved-against-a-real-container* half of
css-contain-3 §6 is not implemented and is not reachable; only the
fallback is.

The fallback base is [`viewport_width`](Self::viewport_width). This
engine models a single viewport size rather than css-values-4's separate
small/large/dynamic viewport sizes, so that one size stands in for the
small viewport.
