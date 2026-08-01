Why a [`DocumentService`] operation failed (spec §6.10).

Each variant is a pure, backend-agnostic value; the informational variants
carry a human-readable reason. This type implements [`std::error::Error`] so
it composes with the standard error ecosystem while remaining I/O-free.
