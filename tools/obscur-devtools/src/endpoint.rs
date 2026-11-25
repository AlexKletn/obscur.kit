/// Where a connection binds/attaches: a loopback TCP address or a Unix path.
pub(super) enum Endpoint {
    /// Loopback TCP. For `serve`, `None` binds an ephemeral port.
    Tcp(Option<String>),
    /// A Unix domain socket path.
    Unix(String),
}
