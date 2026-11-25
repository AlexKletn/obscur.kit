#[doc = include_str!("../../../docs/api/algorithm/grid/auto_repeat.doc/AutoRepeatMode.md")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoRepeatMode {
    /// `repeat(auto-fill, …)`: keep empty repeated tracks (they occupy space).
    AutoFill,
    /// `repeat(auto-fit, …)`: collapse empty repeated tracks to `0` after placement.
    AutoFit,
}
