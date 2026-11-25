DevTools endpoint server: transport, auth, subscriptions, runtime bridge,
and the security-enforcing [`endpoint::DevToolsEndpoint`] (spec §7.6/§7.7).

This crate turns the pure [`obscur_devtools_protocol`] wire types into a
working, in-process endpoint. It is disabled by default, authenticates every
session against a local token registry, negotiates the protocol version,
separates the four capability scopes, enforces subscription/message limits,
audits dangerous operations, and never executes arbitrary code. Two
transports carry the same wire messages: [`transport::LocalTransport`] models
the local socket in process, and [`transport::SocketTransport`] drives the
very same [`endpoint::DevToolsEndpoint`] over a real, local-only kernel
socket (`std::net`/`std::os::unix::net`) using the shared
[`transport::WireCodec`] framing.
