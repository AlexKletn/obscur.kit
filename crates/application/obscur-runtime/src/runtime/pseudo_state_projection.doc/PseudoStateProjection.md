Projects the runtime's live input state — the hover chain, the focused node
and every outstanding press — into the per-node [`PseudoStates`] map the
cascade matches `:hover` / `:focus` / `:active` against (spec §3.10, §4.13).

A borrow bundle over that state, built for one recomputation and dropped with
it. It reads; the caller seats the map it returns.

# Why this is one entity rather than three assignments

Because the three states do not reach equally far, and the map is only correct
if each one reaches exactly as far as CSS says it does:

- `:hover` is the chain the pointer is over, which the caller has already
  resolved and holds as one value;
- `:focus` is **one node**, never its ancestors — focus is where keys go, and
  a container is not focused because something inside it is;
- `:active` is the *ancestor chain* of every outstanding press, which is why a
  press is not one node either: CSS lights the pressed control and everything
  it sits inside.

So one of the three walks nothing, one walks a chain the caller supplies, and
one walks a chain **per press** — and the last is the one that has to be
unioned rather than assigned, because two fingers on two buttons light both
and a node can be hovered and active at once. Written as three loops beside
each other the union discipline is a convention; written here it is the
entity's whole content.
