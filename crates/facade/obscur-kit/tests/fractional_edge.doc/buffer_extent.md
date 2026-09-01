The buffer size the layer runner asks the swapchain for, computed exactly as
`LayerBackend::physical_size` does it: `round(logical * scale)`. At 1.6 that
rounds 89.6 *up* to 90, so the buffer is a shade larger than the surface it
stands for; at 1.2 or 1.4 it rounds down instead, and the buffer is a shade
smaller.
