The set of scopes granted to an endpoint.

Modeled as a small bitset over the four [`PermissionScope`] values. The
server builds this from its auth store during the handshake and the runtime
consults it before executing each command.
