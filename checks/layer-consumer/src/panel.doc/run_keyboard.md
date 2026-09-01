The whole layer entry point, composed.

**Never called, and that is the point.** There is no compositor here; what this
function is for is being *compiled* — in a feature set with no windowing
backend, where the layer runner is the only reader of what boot produces. Every
call in it is one an application on this path actually makes, so a signature or
a `#[cfg]` that stops composing stops the build.

# Errors

Whatever `App::run` returns — including
[`LayerUnsupported`](obscur_kit::AppError::LayerUnsupported) on a desktop with
no layer surfaces.
