Drops the per-URL bookkeeping — parsed vectors, hub handles, referrer
attributions — for every URL this frame's tree and computed styles no longer
mention and no cache still holds.

Every one of those maps is keyed by URL, inserted on first sight and, before
this, emptied only wholesale when a provider was replaced. A file manager
drawing a thumbnail per file accumulated one entry per file it had EVER shown,
for the life of the process.

It is swept in [`mark_image_visibility`](Self::mark_image_visibility) because
that pass already collects exactly the set this needs: the URLs referenced by
a live `background-image` or element `src`, gathered from the one walk of the
computed styles and the tree that the frame already pays for. Fixing a leak by
adding a second whole-tree walk would only trade one cost for another.

The frame order matters and is not incidental: `mark_image_visibility` runs
*before* the three resolve passes, over the same styles and tree they are
about to read. So a URL this sweep drops is one no pass this frame can ask
for, and nothing here can cause a re-request on the next frame that was not
going to happen anyway.

What it does not sweep is the decoded-image cache. An image that scrolled off
screen is deliberately kept — `VisibilityPriority::Hidden` exists to make it
the first victim under pressure, not to delete it on sight — so its byte and
entry budget decides that, and residency holds the URL's handle open so the
cache and the hub cannot disagree about what exists.

One residue is outside this file: a node removed on the incremental restyle
path keeps its entry in the retained `ComputedStyles`, so a `background-image`
URL only that dead node referenced still reads as live here. Element `src`
comes from the tree and has no such hole.
