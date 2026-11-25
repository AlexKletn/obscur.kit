pub(super) enum FrameOutput {
    Straight,
    Premultiplied,
    #[cfg(feature = "lyon")]
    Lyon,
}
