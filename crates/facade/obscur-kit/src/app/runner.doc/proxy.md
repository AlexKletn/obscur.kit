Wakes this loop from the acquire thread.

`Window::request_redraw` marks the window dirty but does not promise to rouse a
loop parked in `wait`, and a loop that skipped its frame for want of a swapchain
image is parked exactly there. A user event through the proxy does both.
