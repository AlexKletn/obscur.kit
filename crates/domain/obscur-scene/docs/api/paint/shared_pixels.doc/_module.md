Pixel bytes shared by refcount, in whichever shape their producer already had
them.

Two shapes rather than one for a measured reason. An image decoder hands over
an `Arc<[u8]>` it already built; a repacked foreign frame hands over the `Vec`
it was just written into. Insisting on either shape makes the other pay for a
second allocation and a full copy of the buffer — at 1080p, six milliseconds a
frame to publish bytes that were already sitting there.

So neither pays. Everything downstream reads
[`as_bytes`](SharedPixels::as_bytes) and never learns which it got.
