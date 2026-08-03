How many pixels one line of a notched wheel scrolls.

It used to live in the winit runner, which multiplied every line report by it
before the runtime saw anything. That put a scrolling POLICY inside a platform
adapter — the last place a reader would look for it — and meant the runtime
could never tell a notched wheel from a trackpad, because by the time a notch
arrived everything was pixels.

Public because a host that windows its own list has to scroll it by the same
amount the runtime would have: `wheel.pixels_y(UiRuntime::WHEEL_LINE_STEP)`.
