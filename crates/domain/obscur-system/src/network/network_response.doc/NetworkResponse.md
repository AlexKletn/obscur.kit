The host loader's response to a [`NetworkRequest`](crate::network::NetworkRequest)
(spec §5.15).

Headers are kept in the order the host reported them (a [`Vec`] of
name/value pairs) because HTTP permits repeated header names; the body is an
[`Arc<[u8]>`] so clones are cheap and the payload can be shared across the
pipeline without copying.
