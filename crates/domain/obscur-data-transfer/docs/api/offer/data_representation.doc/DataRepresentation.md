A single representation of a [`DataOffer`](crate::offer::DataOffer)'s content
in one MIME format (spec §6.20).

An offer typically advertises the same logical content under several MIME
types; each is one `DataRepresentation`. A representation is either *eager* —
its bytes are already in hand, held in an [`Arc<[u8]>`] so clones are cheap
and equal content compares equal — or *lazy*, meaning the bytes are not
materialized yet and must be fetched on demand from a
[`LazyDataProvider`](crate::offer::LazyDataProvider). Modeling async
provision is an adapter concern; this value only records *which* MIME the
representation carries and whether its bytes are inline.
