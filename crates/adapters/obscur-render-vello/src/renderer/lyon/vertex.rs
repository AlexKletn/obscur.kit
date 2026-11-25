use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct Vertex {
    pub(super) position: [f32; 2],
    pub(super) color: [f32; 4],
    // The image pipeline uses these lanes for UV and repeat flags.
    pub(super) shadow: [f32; 4],
    pub(super) shadow_parameters: [f32; 2],
    pub(super) transform: u32,
    pub(super) gradient: u32,
}
