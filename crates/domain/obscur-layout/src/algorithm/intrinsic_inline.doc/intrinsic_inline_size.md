Resolves an intrinsic inline sizing keyword to a concrete content width, or
[`None`] when `width` is not an intrinsic keyword (spec §4.2, CSS Sizing 3
§4.1).

* `min-content` — every child offered **no** inline space, so each shrinks to
  the widest thing it cannot break (its longest word, or its own minimum).
* `max-content` — every child offered **unbounded** inline space, so nothing
  wraps that does not have to.
* `fit-content` — `min(max-content, max(min-content, stretch))`: take what the
  content wants, but neither overflow the space available nor squeeze below
  what cannot break (CSS 2.1 §10.3.5's shrink-to-fit).

`stretch` is the content width this box would have taken under `width: auto`
(the space its container offers), and is used only by `fit-content`.

# This is a content width, so `box-sizing` does not apply

An authored `width` is a border-box length under `box-sizing: border-box`, and
the caller maps it accordingly. An intrinsic keyword authors no length at all:
what comes back from measuring is already the content's own width, so there is
nothing for `box-sizing` to reinterpret. Border and padding are added around it
either way.

# Cost

Each keyword costs one extra measuring pass over the children (`fit-content`
two), on top of the real one. The measure cache memoizes on
`(node, constraints)`, and a probe's constraints differ from the final ones by
construction, so these do not collapse into the final measure — an intrinsic
width genuinely measures its subtree more than once. That is inherent to the
keywords (a browser does the same), and it is why they are resolved only when
authored.
