Feeds a move to `pointer`'s armed pan, reporting the scroll it asks for.

The first delta — the one that crosses the slop — is where a drag stops
being a possible tap and becomes a scroll, so it is where the sequence is
taken away from everything else: the press is **discarded** (the release
will activate nothing), any armed text drag-selection is dropped, and the
pointer is captured to the container. Without the discard, flicking a list
would activate whichever row the finger happened to start on — the single
most noticeable way touch scrolling goes wrong.

What each of the three does is worth keeping straight, because they overlap
and it is easy to credit the wrong one:

- The *remembered* container keeps the drag scrolling the right list once
  the finger has left it.
- The *capture* stops the rest of the sequence interacting with what it
  sweeps across, and — because the release then targets the container
  rather than the row — it is what saves a flick that started **on a row**
  from activating that row.
- The *press discard* covers what capture cannot: a flick that started on
  the container itself, where the pressed node and the captor are the same
  node, the release matches, and the container's own binding would fire at
  the end of every flick.

Points arrive in **content space before each container's own scroll
offset** is folded in. That matters: the scroll-mapped point moves as the
container scrolls, so tracking it would feed the pan its own output.
