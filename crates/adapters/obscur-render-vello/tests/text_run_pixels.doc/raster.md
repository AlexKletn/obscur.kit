Renders a command list offscreen and hands back its straight-alpha RGBA bytes.

This goes through `VelloRenderer::render_to_image`, which is the renderer's own
`render_to_frame` plus a read-back — the same conversion and the same Vello
submit the on-screen present makes.
