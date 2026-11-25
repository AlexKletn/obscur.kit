Owns a swapchain surface and serializes its configuration, acquisition and
optional deferred presentation on a worker thread.

Call [`request`](Self::request) when the window needs an image. The worker
applies the latest requested extent before acquiring, then wakes the window
loop. [`try_take`](Self::try_take) returns the ready image.

A caller can return a rendered texture with [`queue_present`](Self::queue_present)
to keep a blocking buffer swap off the input thread. The next acquisition can
start after the window observes completion. New extent requests replace any
pending extent while the swap is underway.
