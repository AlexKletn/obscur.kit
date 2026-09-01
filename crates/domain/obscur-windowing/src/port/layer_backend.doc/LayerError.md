Why a desktop surface could not be opened.

Three cases, and the first is not a bug: a platform with no desktop-surface
concept answers [`Unsupported`](LayerError::Unsupported), which a host can
branch on to fall back to an ordinary window.
