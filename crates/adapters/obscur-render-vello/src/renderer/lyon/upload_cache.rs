//! Track the bytes most recently uploaded to a GPU buffer.

use std::ops::Range;

use super::packed_upload::PackedUpload;

const BLOCK_BYTES: usize = 256;
const SCAN_BYTES: usize = 16 * BLOCK_BYTES;
const MAX_WRITES: usize = 8;
const MAX_PACKED_BYTES: usize = 64 * 1024;

#[derive(Default)]
pub(super) struct UploadCache {
    bytes: Vec<u8>,
}

impl UploadCache {
    pub(super) fn clear(&mut self) {
        self.bytes.clear();
    }

    pub(super) fn write(
        &mut self,
        queue: &wgpu::Queue,
        packed: &mut PackedUpload,
        buffer: &wgpu::Buffer,
        bytes: &[u8],
    ) -> usize {
        let ranges = changed_ranges(&self.bytes, bytes);
        let total = ranges.clone().map(|range| range.len()).sum::<usize>();
        let pack = packed.len() + total <= MAX_PACKED_BYTES;
        let prefix = self.bytes.len().min(bytes.len());
        self.bytes.truncate(prefix);
        self.bytes.extend_from_slice(&bytes[prefix..]);
        let mut written = 0;
        for range in ranges {
            if pack {
                packed.push(buffer, range.start as u64, &bytes[range.clone()]);
            } else {
                queue.write_buffer(buffer, range.start as u64, &bytes[range.clone()]);
            }
            written += range.len();
            let end = range.end.min(prefix);
            if range.start < end {
                self.bytes[range.start..end].copy_from_slice(&bytes[range.start..end]);
            }
        }
        written
    }
}

fn changed_ranges(
    previous: &[u8],
    next: &[u8],
) -> impl Iterator<Item = Range<usize>> + Clone + use<> {
    let mut ranges = [const { 0..0 }; MAX_WRITES];
    if previous.is_empty() {
        ranges[0] = 0..next.len();
        return ranges.into_iter().take(usize::from(!next.is_empty()));
    }
    let mut start = 0;
    let mut end = next.len();
    // Compare large aligned slices at the edges so small edits only need a
    // fine-grained scan around the changed bytes.
    while start < end {
        let chunk_end = (start + SCAN_BYTES).min(end);
        if previous.get(start..chunk_end) != Some(&next[start..chunk_end]) {
            break;
        }
        start = chunk_end;
    }
    while end > start + SCAN_BYTES {
        let chunk_start = (end - 1) / SCAN_BYTES * SCAN_BYTES;
        if previous.get(chunk_start..end) != Some(&next[chunk_start..end]) {
            break;
        }
        end = chunk_start;
    }
    let mut count = 0;
    for (index, block) in next[start..end].chunks(BLOCK_BYTES).enumerate() {
        let block_start = start + index * BLOCK_BYTES;
        let block_end = block_start + block.len();
        if previous.get(block_start..block_end) == Some(block) {
            continue;
        }
        if count > 0 && ranges[count - 1].end == block_start {
            ranges[count - 1].end = block_end;
        } else if count == MAX_WRITES {
            ranges[0] = ranges[0].start..end;
            count = 1;
            break;
        } else {
            ranges[count] = block_start..block_end;
            count += 1;
        }
    }
    ranges.into_iter().take(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updates_only_changed_blocks_and_handles_resize() {
        let before = vec![0; BLOCK_BYTES * 5];
        let mut after = before.clone();
        after[4] = 1;
        after[BLOCK_BYTES * 4 + 4] = 2;
        assert_eq!(
            changed_ranges(&before, &after).collect::<Vec<_>>(),
            [0..BLOCK_BYTES, BLOCK_BYTES * 4..BLOCK_BYTES * 5]
        );
        let short = &before[..before.len() - 4];
        assert_eq!(changed_ranges(&before, short).count(), 0);
        assert_eq!(
            changed_ranges(short, &before).collect::<Vec<_>>(),
            [BLOCK_BYTES * 4..BLOCK_BYTES * 5]
        );
    }

    #[test]
    fn fragmented_changes_keep_unchanged_edges_out_of_the_upload() {
        let before = vec![0; 4 * SCAN_BYTES];
        let mut after = before.clone();
        for index in (SCAN_BYTES..3 * SCAN_BYTES).step_by(2 * BLOCK_BYTES) {
            after[index] = 1;
        }
        assert_eq!(
            changed_ranges(&before, &after).collect::<Vec<_>>(),
            [SCAN_BYTES..3 * SCAN_BYTES]
        );
    }

    #[test]
    fn fragmented_changes_use_one_write() {
        let before = vec![0; (MAX_WRITES * 2 + 1) * BLOCK_BYTES];
        let mut after = before.clone();
        for index in (0..after.len()).step_by(BLOCK_BYTES * 2) {
            after[index] = 1;
        }
        assert_eq!(
            changed_ranges(&before, &after).collect::<Vec<_>>(),
            [0..after.len()]
        );
    }
}
