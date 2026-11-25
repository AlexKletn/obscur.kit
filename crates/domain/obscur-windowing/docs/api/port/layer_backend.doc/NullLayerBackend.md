A backend for a platform that has no desktop surfaces.

Every operation answers [`Unsupported`](LayerError::Unsupported) and every
feature answers
[`Unavailable`](obscur_capability::SupportLevel::Unavailable). It is the
correct implementation for such a platform — not a stub — and it keeps the port
exercised in builds where no real backend is compiled in.
