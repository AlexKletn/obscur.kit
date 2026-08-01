A handle not yet attached to a surface.

Build one before [`LayerApp::run`](super::LayerApp::run), hand clones to
the handlers that need it, and give one to
[`LayerApp::handle`](super::LayerApp::handle). Until the loop starts it
reports [`LayerSurfaceState::Configuring`] and a zero size.
