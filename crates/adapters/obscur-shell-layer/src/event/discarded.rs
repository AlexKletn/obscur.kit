/// A frame the compositor accepted and never put on the screen.
///
/// The protocol has exactly two answers to a feedback request, and this is the
/// other one. It carries no timestamp because there is nothing to timestamp: the
/// content update this answers was dropped.
///
/// # Why the commit count is here
///
/// `discarded` has one meaning in the protocol — "this content update was not
/// displayed" — and at least two causes that a reader has to tell apart:
///
/// * the surface was not being composited (hidden, animating out, on another
///   workspace), which is nothing to do with the frame; or
/// * a **later commit on the same surface superseded it**, which is entirely to
///   do with the frame, and is a self-inflicted wound if the later commit came
///   from this process.
///
/// So the number of commits this process made on the surface between asking for
/// the feedback and hearing back travels with the answer. Non-zero means we
/// overwrote our own frame before the compositor could show it. Zero means we
/// did not, and the cause is on the other side of the socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Discarded {
    /// Which feedback this answers, counted by the surface that asked.
    pub seq: u64,
    /// Commits this process made on the surface after asking for this feedback.
    ///
    /// Not counting the one the frame itself is: that commit comes from wgpu,
    /// which this side cannot see and does not need to. What is counted is the
    /// *extra* commits — a scale sync, a settings apply — that a frame in flight
    /// does not survive.
    pub commits_since_ask: u64,
}
