Why a [`NetworkService`](super::NetworkService) fetch failed (spec §5.15).

Each variant is a pure, backend-agnostic value; the informational variants
carry a human-readable detail. This type implements [`std::error::Error`] so
it composes with the standard error ecosystem while remaining I/O-free.
