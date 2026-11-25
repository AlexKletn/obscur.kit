/// Protects the brackets that contain an application canvas painter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CanvasBoundary {
    pub(super) depth: u32,
    pub(super) rejected_pop: bool,
}
