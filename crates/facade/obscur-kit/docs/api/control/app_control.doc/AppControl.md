A cheap, clonable handle an application keeps on its own running runtime
(spec §2.6, §4.12, §2.3).

# The problem it solves

`App` never yielded the runtime, and an application therefore
could not reach capabilities the runtime implements and the spec requires:

- §2.6's routing chain is *focused node → focus scope → ancestors → window →
  application fallback*, and only the last link was reachable, because the first
  four are keyed by a [`NodeId`] no application could obtain. One flat command
  namespace was the consequence: the same symbol could not mean different things
  in different places, and a handler could not sit beside the markup it serves.
- §4.12 requires CSS parsing to be "cached per unique source/version and
  invalidated **explicitly**". `App` collected the authored sheets once before
  the loop and nothing ever re-supplied them, so every style edit cost a rebuild.
- §2.3's capture/bubble listeners are per-node too, so they were unreachable for
  the same reason as the routing links.

# The frame-lifecycle contract

This handle does **not** hand out a `&mut UiRuntime`, and that is deliberate.
Application code runs inside the frame — a command handler is called *during*
§2.3's propagation, with the runtime's handler table mutably borrowed by the walk
that is calling it — so touching the runtime there would be re-entrancy, and
retaining it would outlive the borrow that made it available. So every method
here **records** what to do, and the host applies it at one defined point:

> [`apply`](Self::apply) runs once per frame, after the frame has committed and
> before the next one is built.

> [`apply`](Self::apply) runs once per frame, at the frame boundary: before the
> next frame is built, and after the previous one has committed.

Two properties follow, and both matter:

- the tree exists when names are resolved (a `NodeId` is only real after a
  commit), which is what makes positional registration possible at all;
- a request made from a handler is applied after that handler returns, never
  inside it, so nothing an application writes can re-enter the frame.

A queued request is therefore *not* immediate: a stylesheet replaced from a click
handler is installed at the start of the next frame and is visible on that frame.
That is one frame, not a reload.

# Registration and clones

Every clone names one shared queue, so a handler may capture a clone and use it —
which is the point. Methods take `&self` for the same reason.
