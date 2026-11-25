/// A single keyframe offset selector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyframeSelector {
    /// The `from` keyword (equivalent to `0%`).
    From,
    /// The `to` keyword (equivalent to `100%`).
    To,
    /// An explicit percentage offset (for example `50` for `50%`).
    Percent(f64),
}
