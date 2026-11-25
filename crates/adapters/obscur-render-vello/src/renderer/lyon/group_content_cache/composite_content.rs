#[derive(PartialEq)]
pub(super) struct CompositeContent {
    pub(super) revision: u64,
    pub(super) opacity: u32,
    pub(super) bounds: [u32; 4],
    pub(super) stencil_depth: Option<u32>,
}
