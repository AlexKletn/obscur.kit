A client's request to open a DevTools connection.

Carries the versions the client can speak and an opaque authentication
token/capability. Version negotiation and token validation both happen on
the server; this crate only models the envelope.
