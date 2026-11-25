Why a clipboard or drag-and-drop transfer failed (spec §6.20).

Each variant is a pure, backend-agnostic value; the informational variants
carry a human-readable reason describing the specific failure. This type
implements [`std::error::Error`] so it composes with the standard error
ecosystem while remaining I/O-free.
