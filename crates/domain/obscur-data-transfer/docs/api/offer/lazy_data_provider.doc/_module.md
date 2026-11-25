The port for resolving lazy representation bytes on demand (spec §6.20 lazy
async providers).

This module defines [`LazyDataProvider`], the port through which a
[`DataOffer`](crate::offer::DataOffer)'s *lazy*
[representations](crate::offer::DataRepresentation) obtain their bytes when a
consumer finally asks for them. Interface adapters implement it — a clipboard
backend that reads the platform pasteboard, a drag source that serializes an
app value only if a drop actually lands. Lazy provision is inherently async
in real toolkits; the domain models it **synchronously**, so an async adapter
wraps its own future and calls this port when the bytes are ready. The
[`StaticDataProvider`](super::StaticDataProvider) is an honest reference implementation over
an in-memory table.
