An HTTP-like request the host loader fulfils on the kit's behalf
(spec §5.15).

The kit itself opens no sockets: it hands this description to a
host-provided [`NetworkService`](crate::network::NetworkService), which
resolves the URL through its own loader. Headers are kept in insertion order
(a [`Vec`] of name/value pairs) because HTTP permits repeated header names
and preserves their order; the body is an [`Arc<[u8]>`] so clones are cheap
and the bytes can be shared without copying.
