Whether this session will actually deliver pushed events — and therefore
whether a `Subscribe` is granted rather than denied.

False until a serving loop that pushes takes the endpoint over, which is
today only
[`SocketTransport::serve_with_events`](crate::transport::SocketTransport::serve_with_events).
A session driven by [`SocketTransport::serve`](crate::transport::SocketTransport::serve)
stays request/reply and reports `false`.
