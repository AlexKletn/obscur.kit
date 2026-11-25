Why a desktop surface could not be opened.

A platform without desktop surfaces answers [`Unsupported`](LayerError::Unsupported)
so the host can fall back to an ordinary window. When the backend supplies an
underlying error, [`BackendError`](LayerError::BackendError) exposes it through
`std::error::Error::source`.
