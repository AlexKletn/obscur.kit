The host-installed platform accessibility port (spec §7.1) together with the
focus it was last told about.

# Why these two are one entity

Publishing is a single operation that must touch both: it decides *whether*
to push by asking whether a bridge exists **and** whether focus has moved
since the last push, and when it does push it reports the node focus moved
*from* and records the node it moved *to* — in the same call. The recorded
focus is therefore not a fact about the runtime; it is a fact about the
bridge, meaning "what has this port already been told".

Written apart the failure is silent in both directions, which is what makes
this an invariant rather than two neighbouring fields:

- **A push that forgets to record.** The gate `published == focused` never
  goes true again, so every subsequent frame re-walks the whole retained tree
  and re-pushes the same snapshot — precisely the cost
  [`should_publish`](Self::should_publish) exists to avoid, paid at frame
  rate with nothing on screen to show for it. The next push also reports a
  `previous` node that focus already left, so an adapter following focus
  movement is told a transition that did not happen.
- **A record without a push.** The gate then suppresses the push that would
  have carried the move, and the platform is never told focus changed at all.
  Nothing on screen changes either way: the whole failure lives in the output
  path an assistive technology reads and a sighted developer does not.

# What it deliberately does not do

It never builds a semantic tree and never sees the retained tree, the
computed styles or the focus state — [`SemanticTreeBuilder`] builds the
snapshot from the tree and the runtime hands both the snapshot and the
focused node in. That ordering is deliberate rather than incidental: the
snapshot is a full walk of the retained tree, so the gate has to be
answerable *before* anything is built, which is why
[`should_publish`](Self::should_publish) is a separate question from
[`push`](Self::push) rather than a branch inside it.

[`SemanticTreeBuilder`]: crate::runtime::SemanticTreeBuilder
