Applies everything queued and (re)binds every positional registration,
returning how many actions were taken.

**Host contract.** Call at the frame boundary — before a frame is built and
after one has committed — never from inside a handler, and always with the same
[`CssPipeline`] the authored CSS was installed through (the pipeline owns
§4.12's parse cache; a fresh one throws it away).

`App`'s window runner calls it on both edges, and each edge
answers a different need: before the build, so a request queued since the last
frame reaches *this* one; after the commit, so a name resolves against the tree
that now exists. A guest host embedding the runtime does the same in its own
loop. Calling it more often is safe — with nothing queued and every name bound
it is a handful of borrows — but calling it *less* than once per commit delays
a registration by a frame.

Nothing here calls into application code except a
[`with_runtime`](Self::with_runtime) action, and the queue borrow is released
before any of them runs — so an action (or a diagnostic sink reached by a CSS
install) may queue further work, which is applied on the next call rather than
re-entering this one.
