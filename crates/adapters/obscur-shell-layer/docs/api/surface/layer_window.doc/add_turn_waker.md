Let another thread end this loop's wait, without saying anything by it.

Distinct from the waker passed to [`open_with`](Self::open_with), which pushes
[`Woken`](obscur_windowing::layer::LayerSurfaceEvent::Woken) and means
*reconsider the interface*. This one means only *`dispatch` may return now* —
for a thread that has finished something the loop is waiting on and where a
frame request would be wrong.

The case it exists for: a swapchain image acquired off the loop thread. The loop
skipped its frame because no image was ready and is now asleep; when one lands
it must wake and look, and a `Woken` would instead have it rebuild a view that
has not changed.

A waker whose source has already been taken is accepted and does nothing, so
registering twice is not an error.

# Errors

`Dispatch` if the source cannot be inserted into the loop.
