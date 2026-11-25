An opaque authentication token/capability presented by the client.

This crate treats the token as opaque; the server validates it against
its own auth store and must never log it (spec §7.7).
