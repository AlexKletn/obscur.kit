use std::sync::mpsc::Receiver;

pub(super) struct FrameCapture {
    pub(super) buffer: wgpu::Buffer,
    pub(super) ready: Receiver<Result<(), wgpu::BufferAsyncError>>,
    pub(super) id: u64,
    #[cfg(feature = "lyon")]
    pub(super) scene_plan: Option<Vec<[u32; 4]>>,
}
