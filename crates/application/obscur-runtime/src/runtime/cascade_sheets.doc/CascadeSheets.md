The three places a frame's stylesheets come from: the set the reactive path
holds, the last non-empty author set any frame was rendered with, and the
built-in user-agent (UA) default (§4.14).

# Why these three are one entity

They are three *sources* of one list — the ordered `&[Stylesheet]` the
matcher runs on — and resolving that list is a single operation that must
touch two of them together: it substitutes the retained author set when the
frame supplies none, remembers the set when it does, and prepends the UA
sheet below whatever came out. Written as two statements the failure is
silent in both directions:

- **The prepend without the retention**: a frame given no sheets restyles to
  the UA default alone and the author's layout collapses — which is the
  defect [`retained_author`](Self::retained_author) was added for, because a
  pure runtime-state change (a scroll, a hover) re-rendered with `&[]` must
  re-present the *same* styled tree.
- **The retention over the cascade list**: if what is remembered is the list
  the matcher ran on rather than the author half of it, the remembered copy
  carries the UA sheet — and it then **outlives its replacement**. A later
  [`set_default`](Self::set_default) installs a new UA sheet, the next bare
  frame cascades new-UA *and* old-UA, and the stale one wins every tie it
  shares with the new one because it sits later in source order. The UA sheet
  would have become the one thing in the cascade that cannot be replaced.

[`AuthorSheets`] is why that second one cannot be written here: `cascade`
consumes a token only `author_for_frame` can produce, so there is no route
that prepends twice and no route that remembers a prepended list.

The reactive held set belongs with them because it is the third source of the
same list, not a fourth concern: the reactive path *supplies* it to the frame
exactly as a `&View`-taking caller supplies its own slice, so it goes through
the same retention — including the consequence documented on
[`set_reactive`](Self::set_reactive).

# What it deliberately does not do

It never matches anything, never asks for a frame and never sees the tree,
the styles or the matcher: it hands back a `Vec<Stylesheet>` and the runtime
cascades with it. It has no opinion on *precedence* either — that is carried
by each sheet's [`CascadeOrigin`](obscur_style::cascade::CascadeOrigin), and
all this type fixes is the order the matcher sees them in.
