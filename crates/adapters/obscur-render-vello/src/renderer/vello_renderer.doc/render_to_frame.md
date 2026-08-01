Renders `submission` into an offscreen [`VelloFrame`]: the main Vello scene,
then our **own** wgpu gaussian pass for every `filter: blur()` layer.

This is the shared core the on-screen present ([`Self::present_to_surface`]),
the headless read-back ([`Self::render_to_image`]), and the port
[`Renderer::render`] all funnel through, so every path produces the exact
same pixels. The display list is converted into a main scene plus a list of
[`BlurLayer`] sub-scenes ([`SceneConverter::convert_with_blurs`]); the main
scene is rasterized into the frame, then each blur layer is rendered into a
margin-padded offscreen, run through a separable gaussian, and composited
back onto the frame. With no blur layer this is exactly the previous single
[`render_to_texture`](vello::Renderer::render_to_texture) — byte-identical.

# Errors

[`RendererError::DeviceLost`] when no device is attached, or
[`RendererError::Backend`] on a Vello rasterization failure.
