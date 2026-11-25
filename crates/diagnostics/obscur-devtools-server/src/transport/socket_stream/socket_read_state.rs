/// How far the current frame has been read.
#[derive(Debug)]
pub(super) enum SocketReadState {
    /// Reading the `u32` length prefix; `filled` bytes of it have arrived.
    Header { buf: [u8; 4], filled: usize },
    /// The length is known; `filled` bytes of the body have arrived.
    Body { buf: Vec<u8>, filled: usize },
}

impl Default for SocketReadState {
    fn default() -> Self {
        Self::Header {
            buf: [0; 4],
            filled: 0,
        }
    }
}
