Acquires a GPU device and a swapchain for the layer surface.

The same alpha handling as the windowed runner: a premultiplied surface hands
the root's own alpha to the compositor, so a panel is see-through exactly when
its stylesheet says so and there is no mode to opt into.
