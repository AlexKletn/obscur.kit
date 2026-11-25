Borrow a native Wayland connection and surface.

# Safety

The display and surface handles must belong to the same Wayland connection.
The caller must keep the native display and surface alive until the returned
proxies and derived event queues are dropped. In particular, retaining the
Rust window value is sufficient only if its implementation keeps both native
objects alive for that entire period.
