Loads a native guest with an authenticated service session. Service packets
are accepted on the calling thread during binding, initialization, and dispatch.
The loader rejects handles outside the active guest scope and withdraws
the session before deactivation.

# Safety

The trust, ABI, filesystem and handle requirements of [Self::load_with_host]
apply. The host supplies the session for the verified installation.
