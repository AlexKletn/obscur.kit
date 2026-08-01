The reason a resource failed to load, decode, or compile (spec §5.15).

A [`crate::state::ResourceState::Failed`] carries one of these to explain why
the resource could not be made ready. Each variant is a pure, backend-agnostic
value; the informational variants carry a human-readable reason describing the
specific failure. This type implements [`std::error::Error`] so it composes
with the standard error ecosystem, while remaining I/O-free.
