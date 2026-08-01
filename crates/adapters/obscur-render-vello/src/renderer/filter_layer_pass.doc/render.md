Renders each filter sub-scene into its own margin-padded offscreen with
Vello.

The margin is ~3σ of the layer's gaussian, so the blur's tail is not
clipped at the texture edge, and the sub-scene — which draws in **absolute**
coordinates — is translated so its bounds' top-left lands at
`(margin, margin)`.

# Errors

[`RendererError::Backend`] on a Vello rasterization failure.
