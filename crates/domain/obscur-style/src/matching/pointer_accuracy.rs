/// Accuracy of a pointing device available to the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerAccuracy {
    /// Limited targeting precision, such as a finger on a touchscreen.
    Coarse,
    /// Precise targeting, such as a mouse cursor.
    Fine,
}
