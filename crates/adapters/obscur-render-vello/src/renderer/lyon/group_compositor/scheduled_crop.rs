use super::{CropTarget, GroupCrop};

pub(super) struct ScheduledCrop {
    pub(super) region: GroupCrop,
    pub(super) target: CropTarget,
    pub(super) transforms: (wgpu::Buffer, wgpu::BindGroup),
}
