Every live pointer's interaction state: its outstanding press, its click run,
its capture, and the pan, hold or coast it is driving.

# Why these belong to one entity

They are one thing seen at different moments — a pointer's *sequence* — and
they end together. A lift or a cancel has to discard the press, end the hold,
end the pan, release the capture and drop the text drag, and getting any one
of them wrong is a visible bug: a stale hold fires a context action for a
finger that is gone, a stale capture swallows the next gesture, a stale press
activates a control on the following tap. That teardown used to be written out
by hand in the `Up` arm and again in the `Cancel` arm of `handle_pointer`,
four steps each, differing only in `end()` versus `cancel()`. It is
[`end_pointer`](Self::end_pointer) now, once.

# What it deliberately does not do

It never hit-tests, never scrolls and never asks for a frame. The recognizers
here answer *what the contact is doing*; the caller resolves which container a
drag belongs to, applies the scroll, and decides which frames that is worth.
So [`update_pan`](Self::update_pan) returns a [`PanStep`] instead of
scrolling, and [`advance_flings`](Self::advance_flings) returns distances
instead of applying them.
