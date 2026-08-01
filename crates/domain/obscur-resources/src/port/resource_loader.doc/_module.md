The hexagonal port for loading a resource's raw encoded bytes (spec §5.15).

This module defines [`ResourceLoader`], the port through which the pure
resource domain obtains the raw *encoded* bytes behind a [`ResourceSource`]
(spec §5.15 custom loaders). Interface adapters implement it — an images/font
crate that opens files, a host loader that fetches URLs or reads picked
documents — while the kit itself performs no network access and reads no host
documents by itself. The colocated [`LoadedBytes`] output pairs the encoded
bytes with their [`ResourceMetadata`]; the colocated [`InlineLoader`] is an
honest reference implementation that resolves only the bytes-in-hand sources.

Loading is modeled **synchronously** at the domain level: an adapter that
loads asynchronously wraps its own async and calls this port when the bytes
are ready.
