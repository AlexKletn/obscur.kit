/// A call decoded from a stream frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServiceCallFrame<'a> {
    /// Numeric handle previously issued to this connection.
    pub handle: u64,
    /// Requested service operation.
    pub operation: &'a str,
    /// Encoded JSON request.
    pub payload: &'a [u8],
}
