use std::ops::Range;

#[derive(Default)]
pub(super) struct PackedUpload {
    buffer: Option<wgpu::Buffer>,
    bytes: Vec<u8>,
    copies: Vec<(wgpu::Buffer, u64, Range<u64>)>,
}

impl PackedUpload {
    pub(super) fn len(&self) -> usize {
        self.bytes.len()
    }

    pub(super) fn push(&mut self, target: &wgpu::Buffer, offset: u64, bytes: &[u8]) {
        let start = self.bytes.len() as u64;
        self.bytes.extend_from_slice(bytes);
        self.copies
            .push((target.clone(), offset, start..self.bytes.len() as u64));
    }

    pub(super) fn encode(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if self.bytes.is_empty() {
            return;
        }
        let size = self.bytes.len() as u64;
        if self
            .buffer
            .as_ref()
            .is_none_or(|buffer| buffer.size() < size)
        {
            self.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("obscur-lyon-packed-upload"),
                size: size.next_power_of_two(),
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        let source = self.buffer.as_ref().expect("upload buffer");
        queue.write_buffer(source, 0, &self.bytes);
        for (target, offset, source_range) in self.copies.drain(..) {
            encoder.copy_buffer_to_buffer(
                source,
                source_range.start,
                &target,
                offset,
                source_range.end - source_range.start,
            );
        }
        self.bytes.clear();
    }
}
