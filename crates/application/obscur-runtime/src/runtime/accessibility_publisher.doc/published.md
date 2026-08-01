The focus last published to [`bridge`](Self::bridge), so a frame can tell
whether focus moved since — and so the next push reports the node it
actually moved *from*.

One record, not one per bridge. [`push`](Self::push) advances it for
*whatever* bridge it was handed, including a host's own — see that
method for the cross-talk that buys.
