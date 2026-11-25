//! Keep GPU mesh buffers and update their changed byte ranges.

use super::packed_upload::PackedUpload;
use super::upload_cache::UploadCache;

#[derive(Default)]
pub(super) struct MeshBuffer {
    buffer: Option<wgpu::Buffer>,
    uploaded: UploadCache,
}

impl MeshBuffer {
    pub(super) fn buffer(&self) -> Option<&wgpu::Buffer> {
        self.buffer.as_ref()
    }

    pub(super) fn upload(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        packed: &mut PackedUpload,
        usage: wgpu::BufferUsages,
        bytes: &[u8],
    ) -> usize {
        if bytes.is_empty() {
            return 0;
        }
        let size = bytes.len() as u64;
        let limit = device.limits().max_buffer_size;
        debug_assert!(size <= limit);
        if self
            .buffer
            .as_ref()
            .is_none_or(|buffer| buffer.size() < size)
        {
            self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("obscur-lyon-mesh"),
                size: size.next_power_of_two().min(limit),
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.uploaded.clear();
        }
        self.uploaded.write(
            queue,
            packed,
            self.buffer.as_ref().expect("mesh buffer"),
            bytes,
        )
    }
}
