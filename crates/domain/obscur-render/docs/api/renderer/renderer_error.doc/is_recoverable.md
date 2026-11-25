Whether the caller can continue after this error.

A [`RendererError::DeviceLost`] is recoverable exactly when its
[`DeviceLoss::is_recoverable`] flag is set (the runtime may rebuild the
device and retry). A [`RendererError::UnsupportedEffect`] is always
recoverable: it names one effect the backend will not reproduce, never a
failure of the operation, so a caller may drop that effect and re-render (and
where a backend raises it as a diagnostic, the frame was drawn regardless).
The remaining variants are terminal for the operation.
