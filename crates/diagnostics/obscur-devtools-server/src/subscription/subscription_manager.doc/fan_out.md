Builds one [`ProtocolEvent`] per subscriber of `topic`, carrying `kind`.

Subscribers of other topics receive nothing. The returned events are in
subscription insertion order.
